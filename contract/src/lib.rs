//! Surety: member-guaranteed loans on top of an existing Sputnik DAO v2.
//!
//! Custody of the loan pool stays in the DAO treasury. This contract only:
//! - holds members' savings (which guarantors lock behind loans),
//! - files a normal Sputnik `Transfer` proposal once a loan is fully guaranteed,
//! - forwards repayments to the DAO treasury,
//! - on default, moves the guarantors' locked savings to the DAO treasury (`claim`),
//! - keeps each member's repayment record and derives their borrowing limit from it.

use near_sdk::json_types::{U128, U64};
use near_sdk::serde_json::{json, Value};
use near_sdk::store::{IterableMap, Vector};
use near_sdk::{
    env, ext_contract, near, require, AccountId, Gas, NearToken, PanicOnDefault, Promise,
    PromiseError,
};

const GAS_ADD_PROPOSAL: Gas = Gas::from_tgas(30);
const GAS_GET_PROPOSAL: Gas = Gas::from_tgas(10);
const GAS_CALLBACK: Gas = Gas::from_tgas(15);
const MIN_FIRST_DEPOSIT: NearToken = NearToken::from_millinear(10);
const NS_PER_SEC: u64 = 1_000_000_000;

#[near(serializers = [json])]
pub struct ProposalInput {
    pub description: String,
    pub kind: Value,
}

#[allow(dead_code)]
#[ext_contract(ext_dao)]
trait SputnikDao {
    fn add_proposal(&mut self, proposal: ProposalInput) -> u64;
    fn get_proposal(&self, id: u64) -> Value;
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug, PartialEq)]
pub enum LoanStatus {
    /// Requested; collecting guarantor pledges.
    Open,
    /// add_proposal call in flight.
    Filing,
    /// Sputnik Transfer proposal filed; waiting for council vote.
    Filed,
    /// Proposal approved and paid out by the DAO; repayment clock running.
    Active,
    /// Fully repaid to the DAO treasury; pledges released.
    Repaid,
    /// Past due and claimed; guarantors' pledges sent to the DAO treasury.
    Defaulted,
    /// Withdrawn by borrower, or proposal rejected / removed / expired.
    Cancelled,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug)]
pub struct Pledge {
    pub guarantor: AccountId,
    pub amount: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug)]
pub struct Loan {
    pub id: u64,
    pub borrower: AccountId,
    pub amount: U128,
    pub duration_sec: U64,
    pub purpose: String,
    pub status: LoanStatus,
    pub pledges: Vec<Pledge>,
    pub proposal_id: Option<u64>,
    pub bond_payer: Option<AccountId>,
    pub bond: U128,
    pub created_at: U64,
    pub disbursed_at: Option<U64>,
    pub due_at: Option<U64>,
    pub repaid: U128,
    pub claimed: U128,
}

impl Loan {
    fn pledged(&self) -> u128 {
        self.pledges.iter().map(|p| p.amount.0).sum()
    }
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug, Default)]
pub struct Member {
    /// Free savings (withdrawable, pledgeable).
    pub savings: U128,
    /// Savings locked behind other members' loans.
    pub locked: U128,
    pub loans_repaid: u32,
    pub loans_defaulted: u32,
    pub repaid_volume: U128,
    /// Guarantees that ended because the borrower repaid.
    pub guarantees_released: u32,
    /// Guarantees that were claimed after a default.
    pub guarantees_claimed: u32,
    pub claimed_from_me: U128,
}

#[near(serializers = [json])]
pub struct MemberView {
    pub account_id: AccountId,
    pub limit: U128,
    #[serde(flatten)]
    pub member: Member,
}

#[near(serializers = [json])]
pub struct ConfigView {
    pub dao: AccountId,
    pub base_limit: U128,
    pub limit_step: U128,
    pub min_guarantors: u32,
    pub loan_count: u64,
    pub member_count: u32,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Surety {
    dao: AccountId,
    base_limit: u128,
    limit_step: u128,
    min_guarantors: u32,
    members: IterableMap<AccountId, Member>,
    loans: Vector<Loan>,
}

fn emit(event: &str, data: Value) {
    env::log_str(&format!(
        "EVENT_JSON:{}",
        json!({"standard": "surety", "version": "1.0.0", "event": event, "data": [data]})
    ));
}

/// A NEAR named account (e.g. `borrower-a.testnet`), not a 64-hex implicit or 0x eth-implicit account.
pub fn is_named(account: &AccountId) -> bool {
    let s = account.as_str();
    let hex64 = s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit());
    let eth = s.len() == 42 && s.starts_with("0x");
    !hex64 && !eth
}

/// floor(a * b / d) without u128 overflow.
pub fn mul_div(a: u128, b: u128, d: u128) -> u128 {
    assert!(d > 0, "div by zero");
    const MASK: u128 = u64::MAX as u128;
    let (a_hi, a_lo) = (a >> 64, a & MASK);
    let (b_hi, b_lo) = (b >> 64, b & MASK);
    let ll = a_lo * b_lo;
    let lh = a_lo * b_hi;
    let hl = a_hi * b_lo;
    let hh = a_hi * b_hi;
    let mid = (ll >> 64) + (lh & MASK) + (hl & MASK);
    let lo = (ll & MASK) | (mid << 64);
    let hi = hh + (lh >> 64) + (hl >> 64) + (mid >> 64);
    let (mut rem, mut q) = (0u128, 0u128);
    for i in (0..256u32).rev() {
        let bit = if i >= 128 { (hi >> (i - 128)) & 1 } else { (lo >> i) & 1 };
        let carry = rem >> 127;
        rem = (rem << 1) | bit;
        if carry == 1 || rem >= d {
            rem = rem.wrapping_sub(d);
            if i < 128 {
                q |= 1u128 << i;
            } else {
                panic!("mul_div overflow");
            }
        }
    }
    q
}

#[near]
impl Surety {
    /// `dao`: the Sputnik DAO v2 whose treasury funds loans.
    /// limit(member) = 0 after any default, else base_limit + limit_step * (loans_repaid + guarantees_released).
    #[init]
    pub fn new(dao: AccountId, base_limit: U128, limit_step: U128, min_guarantors: u32) -> Self {
        require!(min_guarantors >= 1, "min_guarantors must be >= 1");
        Self {
            dao,
            base_limit: base_limit.0,
            limit_step: limit_step.0,
            min_guarantors,
            members: IterableMap::new(b"m"),
            loans: Vector::new(b"l"),
        }
    }

    // ---------- savings ----------

    #[payable]
    pub fn deposit(&mut self) -> U128 {
        let who = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let is_new = !self.members.contains_key(&who);
        if is_new {
            require!(amount >= MIN_FIRST_DEPOSIT, "first deposit must be >= 0.01 NEAR");
        }
        require!(!amount.is_zero(), "attach NEAR to deposit");
        let m = self.members.entry(who.clone()).or_default();
        m.savings = U128(m.savings.0 + amount.as_yoctonear());
        let savings = m.savings;
        emit("deposit", json!({"member": who, "amount": U128(amount.as_yoctonear()), "savings": savings}));
        savings
    }

    pub fn withdraw(&mut self, amount: U128) -> Promise {
        let who = env::predecessor_account_id();
        let m = self.members.get_mut(&who).expect("not a member");
        require!(amount.0 > 0 && amount.0 <= m.savings.0, "amount exceeds free savings");
        m.savings = U128(m.savings.0 - amount.0);
        emit("withdraw", json!({"member": who, "amount": amount}));
        Promise::new(who).transfer(NearToken::from_yoctonear(amount.0))
    }

    // ---------- loan lifecycle ----------

    pub fn request_loan(&mut self, amount: U128, duration_sec: U64, purpose: Option<String>) -> u64 {
        let borrower = env::predecessor_account_id();
        require!(is_named(&borrower), "borrower must be a named account");
        require!(self.members.contains_key(&borrower), "deposit savings first");
        require!(amount.0 > 0, "amount must be > 0");
        let limit = self.limit_of(&borrower);
        require!(amount.0 <= limit, format!("amount exceeds your limit of {} yocto", limit));
        require!(duration_sec.0 > 0, "duration must be > 0");
        let busy = self.loans.iter().any(|l| {
            l.borrower == borrower
                && matches!(l.status, LoanStatus::Open | LoanStatus::Filing | LoanStatus::Filed | LoanStatus::Active)
        });
        require!(!busy, "you already have a loan in progress");
        let purpose = purpose.unwrap_or_default();
        require!(purpose.len() <= 140, "purpose too long");
        let id = self.loans.len() as u64;
        self.loans.push(Loan {
            id,
            borrower: borrower.clone(),
            amount,
            duration_sec,
            purpose,
            status: LoanStatus::Open,
            pledges: vec![],
            proposal_id: None,
            bond_payer: None,
            bond: U128(0),
            created_at: U64(env::block_timestamp()),
            disbursed_at: None,
            due_at: None,
            repaid: U128(0),
            claimed: U128(0),
        });
        emit("loan_requested", json!({"loan_id": id, "borrower": borrower, "amount": amount, "duration_sec": duration_sec}));
        id
    }

    /// Guarantor locks `amount` of their free savings behind `loan_id`.
    pub fn pledge(&mut self, loan_id: u64, amount: U128) {
        let g = env::predecessor_account_id();
        require!(is_named(&g), "guarantor must be a named account");
        let loan = self.loans.get(loan_id as u32).expect("no such loan").clone();
        require!(loan.status == LoanStatus::Open, "loan is not collecting pledges");
        require!(g != loan.borrower, "borrower cannot guarantee own loan");
        require!(amount.0 > 0, "amount must be > 0");
        require!(loan.pledged() + amount.0 <= loan.amount.0, "pledge exceeds remaining cover");
        let m = self.members.get_mut(&g).expect("deposit savings first");
        require!(m.savings.0 >= amount.0, "not enough free savings");
        m.savings = U128(m.savings.0 - amount.0);
        m.locked = U128(m.locked.0 + amount.0);
        let loan = self.loans.get_mut(loan_id as u32).unwrap();
        match loan.pledges.iter_mut().find(|p| p.guarantor == g) {
            Some(p) => p.amount = U128(p.amount.0 + amount.0),
            None => loan.pledges.push(Pledge { guarantor: g.clone(), amount }),
        }
        let covered = U128(loan.pledged());
        emit("pledged", json!({"loan_id": loan_id, "guarantor": g, "amount": amount, "covered": covered, "needed": loan.amount}));
    }

    /// Guarantor withdraws their pledge while the loan is still Open.
    pub fn unpledge(&mut self, loan_id: u64) {
        let g = env::predecessor_account_id();
        let loan = self.loans.get_mut(loan_id as u32).expect("no such loan");
        require!(loan.status == LoanStatus::Open, "pledges are locked once filed");
        let idx = loan.pledges.iter().position(|p| p.guarantor == g).expect("no pledge");
        let p = loan.pledges.remove(idx);
        let m = self.members.get_mut(&g).unwrap();
        m.locked = U128(m.locked.0 - p.amount.0);
        m.savings = U128(m.savings.0 + p.amount.0);
        emit("unpledged", json!({"loan_id": loan_id, "guarantor": g, "amount": p.amount}));
    }

    pub fn cancel_loan(&mut self, loan_id: u64) {
        let loan = self.loans.get(loan_id as u32).expect("no such loan").clone();
        require!(env::predecessor_account_id() == loan.borrower, "only borrower");
        require!(loan.status == LoanStatus::Open, "only open loans can be cancelled");
        self.release_pledges(&loan, false);
        self.loans.get_mut(loan_id as u32).unwrap().status = LoanStatus::Cancelled;
        emit("loan_cancelled", json!({"loan_id": loan_id}));
    }

    /// Once fully covered by >= min_guarantors, file a Sputnik Transfer proposal
    /// (DAO treasury -> borrower). Attach exactly the DAO's proposal bond; it is
    /// refunded to the caller when the proposal is approved, rejected or expired.
    #[payable]
    pub fn file_proposal(&mut self, loan_id: u64) -> Promise {
        let bond = env::attached_deposit();
        let me = env::current_account_id();
        let loan = self.loans.get_mut(loan_id as u32).expect("no such loan");
        require!(loan.status == LoanStatus::Open, "loan is not open");
        require!(loan.pledged() == loan.amount.0, "loan is not fully guaranteed");
        require!(loan.pledges.len() as u32 >= self.min_guarantors, "not enough guarantors");
        loan.status = LoanStatus::Filing;
        loan.bond_payer = Some(env::predecessor_account_id());
        loan.bond = U128(bond.as_yoctonear());
        let guarantors: Vec<String> = loan.pledges.iter().map(|p| p.guarantor.to_string()).collect();
        let description = format!(
            "Surety loan #{} for {}: {} yoctoNEAR, guaranteed by {}. Repay within {}s of payout via {}. ref surety:{}:loan:{}",
            loan.id,
            loan.borrower,
            loan.amount.0,
            guarantors.join(" + "),
            loan.duration_sec.0,
            me,
            me,
            loan.id
        );
        let kind = json!({"Transfer": {
            "token_id": "",
            "receiver_id": loan.borrower,
            "amount": loan.amount,
            "msg": Value::Null
        }});
        ext_dao::ext(self.dao.clone())
            .with_attached_deposit(bond)
            .with_static_gas(GAS_ADD_PROPOSAL)
            .add_proposal(ProposalInput { description, kind })
            .then(Self::ext(me).with_static_gas(GAS_CALLBACK).on_proposal_filed(loan_id))
    }

    #[private]
    pub fn on_proposal_filed(&mut self, #[callback_result] r: Result<u64, PromiseError>, loan_id: u64) -> Option<u64> {
        self.internal_on_filed(loan_id, r.ok())
    }

    /// Anyone: read the Sputnik proposal and advance the loan
    /// (Approved -> Active; Rejected/Expired/Removed -> Cancelled).
    pub fn sync(&mut self, loan_id: u64) -> Promise {
        let loan = self.loans.get(loan_id as u32).expect("no such loan");
        require!(loan.status == LoanStatus::Filed, "loan is not waiting on a proposal");
        let pid = loan.proposal_id.unwrap();
        ext_dao::ext(self.dao.clone())
            .with_static_gas(GAS_GET_PROPOSAL)
            .get_proposal(pid)
            .then(Self::ext(env::current_account_id()).with_static_gas(GAS_CALLBACK).on_proposal_status(loan_id))
    }

    #[private]
    pub fn on_proposal_status(&mut self, #[callback_result] r: Result<Value, PromiseError>, loan_id: u64) -> String {
        let v = r.expect("could not read proposal");
        let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("Unknown").to_string();
        self.internal_on_status(loan_id, &status);
        status
    }

    /// Repay (part of) an active loan. Funds go straight to the DAO treasury.
    #[payable]
    pub fn repay(&mut self, loan_id: u64) -> U128 {
        let payer = env::predecessor_account_id();
        let attached = env::attached_deposit().as_yoctonear();
        let loan = self.loans.get(loan_id as u32).expect("no such loan").clone();
        require!(loan.status == LoanStatus::Active, "loan is not active");
        let outstanding = loan.amount.0 - loan.repaid.0;
        let pay = attached.min(outstanding);
        require!(pay > 0, "attach NEAR to repay");
        let change = attached - pay;
        let new_repaid = loan.repaid.0 + pay;
        {
            let l = self.loans.get_mut(loan_id as u32).unwrap();
            l.repaid = U128(new_repaid);
        }
        Promise::new(self.dao.clone()).transfer(NearToken::from_yoctonear(pay)).detach();
        if change > 0 {
            Promise::new(payer.clone()).transfer(NearToken::from_yoctonear(change)).detach();
        }
        emit("repaid", json!({"loan_id": loan_id, "payer": payer, "amount": U128(pay), "to": self.dao, "outstanding": U128(outstanding - pay)}));
        if new_repaid == loan.amount.0 {
            self.release_pledges(&loan, true);
            let b = self.members.get_mut(&loan.borrower).unwrap();
            b.loans_repaid += 1;
            b.repaid_volume = U128(b.repaid_volume.0 + loan.amount.0);
            self.loans.get_mut(loan_id as u32).unwrap().status = LoanStatus::Repaid;
            let new_limit = U128(self.limit_of(&loan.borrower));
            emit("loan_repaid", json!({"loan_id": loan_id, "borrower": loan.borrower, "new_limit": new_limit}));
        }
        U128(outstanding - pay)
    }

    /// Anyone, after the due date: move the guarantors' locked savings covering
    /// the outstanding balance to the DAO treasury, pro rata to their pledges.
    pub fn claim(&mut self, loan_id: u64) -> U128 {
        let loan = self.loans.get(loan_id as u32).expect("no such loan").clone();
        require!(loan.status == LoanStatus::Active, "loan is not active");
        let due = loan.due_at.unwrap().0;
        require!(env::block_timestamp() > due, "loan is not past due");
        let outstanding = loan.amount.0 - loan.repaid.0;
        let total_pledged = loan.pledged();
        let mut shares: Vec<u128> = loan
            .pledges
            .iter()
            .map(|p| mul_div(p.amount.0, outstanding, total_pledged))
            .collect();
        let mut rem = outstanding - shares.iter().sum::<u128>();
        for (i, p) in loan.pledges.iter().enumerate() {
            if rem == 0 {
                break;
            }
            if shares[i] < p.amount.0 {
                shares[i] += 1;
                rem -= 1;
            }
        }
        let mut taken = vec![];
        for (p, share) in loan.pledges.iter().zip(shares.iter()) {
            let m = self.members.get_mut(&p.guarantor).unwrap();
            m.locked = U128(m.locked.0 - p.amount.0);
            m.savings = U128(m.savings.0 + (p.amount.0 - share));
            m.guarantees_claimed += 1;
            m.claimed_from_me = U128(m.claimed_from_me.0 + share);
            taken.push(json!({"guarantor": p.guarantor, "amount": U128(*share)}));
        }
        self.members.get_mut(&loan.borrower).unwrap().loans_defaulted += 1;
        {
            let l = self.loans.get_mut(loan_id as u32).unwrap();
            l.status = LoanStatus::Defaulted;
            l.claimed = U128(outstanding);
        }
        Promise::new(self.dao.clone()).transfer(NearToken::from_yoctonear(outstanding)).detach();
        emit("claimed", json!({"loan_id": loan_id, "borrower": loan.borrower, "amount": U128(outstanding), "to": self.dao, "from": taken}));
        U128(outstanding)
    }

    // ---------- views ----------

    pub fn get_config(&self) -> ConfigView {
        ConfigView {
            dao: self.dao.clone(),
            base_limit: U128(self.base_limit),
            limit_step: U128(self.limit_step),
            min_guarantors: self.min_guarantors,
            loan_count: self.loans.len() as u64,
            member_count: self.members.len(),
        }
    }

    pub fn get_member(&self, account_id: AccountId) -> Option<MemberView> {
        self.members.get(&account_id).map(|m| MemberView {
            account_id: account_id.clone(),
            limit: U128(self.limit_of(&account_id)),
            member: m.clone(),
        })
    }

    pub fn get_members(&self, from_index: Option<u32>, limit: Option<u32>) -> Vec<MemberView> {
        self.members
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(100) as usize)
            .map(|(a, m)| MemberView { account_id: a.clone(), limit: U128(self.limit_of(a)), member: m.clone() })
            .collect()
    }

    pub fn get_loan(&self, loan_id: u64) -> Option<Loan> {
        self.loans.get(loan_id as u32).cloned()
    }

    pub fn get_loans(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Loan> {
        self.loans
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(100) as usize)
            .cloned()
            .collect()
    }

    pub fn get_limit(&self, account_id: AccountId) -> U128 {
        U128(self.limit_of(&account_id))
    }
}

impl Surety {
    fn limit_of(&self, a: &AccountId) -> u128 {
        match self.members.get(a) {
            None => 0,
            Some(m) if m.loans_defaulted > 0 => 0,
            Some(m) => self.base_limit + self.limit_step * (m.loans_repaid as u128 + m.guarantees_released as u128),
        }
    }

    fn release_pledges(&mut self, loan: &Loan, honored: bool) {
        for p in &loan.pledges {
            let m = self.members.get_mut(&p.guarantor).unwrap();
            m.locked = U128(m.locked.0 - p.amount.0);
            m.savings = U128(m.savings.0 + p.amount.0);
            if honored {
                m.guarantees_released += 1;
            }
        }
    }

    fn refund_bond(&mut self, loan_id: u64) {
        let l = self.loans.get(loan_id as u32).unwrap();
        if let (Some(payer), b) = (l.bond_payer.clone(), l.bond.0) {
            if b > 0 {
                Promise::new(payer).transfer(NearToken::from_yoctonear(b)).detach();
            }
        }
    }

    pub(crate) fn internal_on_filed(&mut self, loan_id: u64, proposal_id: Option<u64>) -> Option<u64> {
        let loan = self.loans.get_mut(loan_id as u32).unwrap();
        require!(loan.status == LoanStatus::Filing, "unexpected state");
        match proposal_id {
            Some(pid) => {
                loan.status = LoanStatus::Filed;
                loan.proposal_id = Some(pid);
                emit("proposal_filed", json!({"loan_id": loan_id, "dao": self.dao, "proposal_id": pid}));
            }
            None => {
                // DAO refunded the attached bond to us; return it to the payer.
                loan.status = LoanStatus::Open;
                self.refund_bond(loan_id);
                let l = self.loans.get_mut(loan_id as u32).unwrap();
                l.bond_payer = None;
                l.bond = U128(0);
                emit("proposal_failed", json!({"loan_id": loan_id}));
            }
        }
        proposal_id
    }

    pub(crate) fn internal_on_status(&mut self, loan_id: u64, status: &str) {
        let loan = self.loans.get(loan_id as u32).unwrap().clone();
        if loan.status != LoanStatus::Filed {
            return;
        }
        match status {
            "Approved" => {
                let now = env::block_timestamp();
                let l = self.loans.get_mut(loan_id as u32).unwrap();
                l.status = LoanStatus::Active;
                l.disbursed_at = Some(U64(now));
                l.due_at = Some(U64(now + loan.duration_sec.0 * NS_PER_SEC));
                let due = l.due_at;
                self.refund_bond(loan_id);
                emit("loan_active", json!({"loan_id": loan_id, "due_at": due}));
            }
            "Rejected" | "Expired" | "Removed" => {
                self.release_pledges(&loan, false);
                if status != "Removed" {
                    self.refund_bond(loan_id);
                }
                self.loans.get_mut(loan_id as u32).unwrap().status = LoanStatus::Cancelled;
                emit("loan_cancelled", json!({"loan_id": loan_id, "proposal_status": status}));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
