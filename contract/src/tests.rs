use super::*;
use near_sdk::test_utils::{get_logs, VMContextBuilder};
use near_sdk::testing_env;

const NEAR: u128 = 1_000_000_000_000_000_000_000_000;

fn acc(s: &str) -> AccountId {
    s.parse().unwrap()
}

fn ctx(who: &str, deposit: u128, ts_sec: u64) {
    let mut b = VMContextBuilder::new();
    b.current_account_id(acc("surety.testnet"))
        .predecessor_account_id(acc(who))
        .attached_deposit(NearToken::from_yoctonear(deposit))
        .block_timestamp(ts_sec * NS_PER_SEC)
        .account_balance(NearToken::from_near(100));
    testing_env!(b.build());
}

fn setup() -> Surety {
    ctx("owner.testnet", 0, 0);
    let mut c = Surety::new(acc("coop.sputnik-v2.testnet"), U128(2 * NEAR), U128(NEAR), 2);
    for (who, amt) in [("ada.testnet", 1), ("bola.testnet", 3), ("chidi.testnet", 3), ("dayo.testnet", 1)] {
        ctx(who, amt * NEAR, 0);
        c.deposit();
    }
    c
}

/// request + two pledges + file + approve, returns loan id.
fn activate(c: &mut Surety, borrower: &str, amount: u128, duration: u64, t: u64) -> u64 {
    ctx(borrower, 0, t);
    let id = c.request_loan(U128(amount), U64(duration), Some("stock".into()));
    ctx("bola.testnet", 0, t);
    c.pledge(id, U128(amount / 2));
    ctx("chidi.testnet", 0, t);
    c.pledge(id, U128(amount - amount / 2));
    ctx(borrower, NEAR / 10, t);
    let _ = c.file_proposal(id);
    assert_eq!(c.get_loan(id).unwrap().status, LoanStatus::Filing);
    c.internal_on_filed(id, Some(7 + id));
    assert_eq!(c.get_loan(id).unwrap().status, LoanStatus::Filed);
    ctx("anyone.testnet", 0, t + 10);
    c.internal_on_status(id, "Approved");
    id
}

#[test]
fn mul_div_works() {
    assert_eq!(mul_div(10, 20, 3), 66);
    let big = 1_000_000 * NEAR; // 1e30
    assert_eq!(mul_div(big, big, big), big);
    assert_eq!(mul_div(u128::MAX, u128::MAX, u128::MAX), u128::MAX);
    assert_eq!(mul_div(3 * NEAR, 2 * NEAR, 4 * NEAR), 3 * NEAR / 2);
}

#[test]
fn named_accounts() {
    assert!(is_named(&acc("ada.testnet")));
    assert!(!is_named(&acc(&"a".repeat(64))));
    assert!(!is_named(&acc("0x1234567890123456789012345678901234567890")));
}

#[test]
fn deposit_and_limits() {
    let c = setup();
    let m = c.get_member(acc("ada.testnet")).unwrap();
    assert_eq!(m.member.savings.0, NEAR);
    assert_eq!(m.limit.0, 2 * NEAR);
    assert!(c.get_member(acc("nobody.testnet")).is_none());
}

#[test]
#[should_panic(expected = "exceeds your limit")]
fn request_over_limit_fails() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    c.request_loan(U128(3 * NEAR), U64(60), None);
}

#[test]
#[should_panic(expected = "borrower cannot guarantee own loan")]
fn self_pledge_fails() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    c.pledge(id, U128(NEAR / 2));
}

#[test]
#[should_panic(expected = "not fully guaranteed")]
fn file_requires_full_cover() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    ctx("ada.testnet", NEAR / 10, 0);
    let _ = c.file_proposal(id);
}

#[test]
#[should_panic(expected = "not enough guarantors")]
fn file_requires_two_guarantors() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(NEAR));
    ctx("ada.testnet", NEAR / 10, 0);
    let _ = c.file_proposal(id);
}

#[test]
#[should_panic(expected = "not enough free savings")]
fn pledge_needs_savings() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(2 * NEAR), U64(60), None);
    ctx("dayo.testnet", 0, 0);
    c.pledge(id, U128(2 * NEAR));
}

#[test]
fn pledge_locks_and_unpledge_releases() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    let b = c.get_member(acc("bola.testnet")).unwrap().member;
    assert_eq!((b.savings.0, b.locked.0), (3 * NEAR - NEAR / 2, NEAR / 2));
    c.unpledge(id);
    let b = c.get_member(acc("bola.testnet")).unwrap().member;
    assert_eq!((b.savings.0, b.locked.0), (3 * NEAR, 0));
}

#[test]
fn repaid_loan_raises_limits() {
    let mut c = setup();
    let id = activate(&mut c, "ada.testnet", NEAR, 60, 0);
    let l = c.get_loan(id).unwrap();
    assert_eq!(l.status, LoanStatus::Active);
    assert_eq!(l.due_at.unwrap().0, 70 * NS_PER_SEC);
    // withdraw of locked savings impossible: bola has 2.5 free
    ctx("ada.testnet", NEAR / 4, 20);
    assert_eq!(c.repay(id).0, 3 * NEAR / 4);
    // overpay is capped; change refunded
    ctx("ada.testnet", 2 * NEAR, 30);
    assert_eq!(c.repay(id).0, 0);
    assert!(get_logs().iter().any(|l| l.contains("loan_repaid")));
    assert_eq!(c.get_loan(id).unwrap().status, LoanStatus::Repaid);
    let a = c.get_member(acc("ada.testnet")).unwrap();
    assert_eq!(a.member.loans_repaid, 1);
    assert_eq!(a.limit.0, 3 * NEAR);
    let b = c.get_member(acc("bola.testnet")).unwrap();
    assert_eq!((b.member.locked.0, b.member.savings.0), (0, 3 * NEAR));
    assert_eq!(b.limit.0, 3 * NEAR);
    // limit actually usable now
    ctx("ada.testnet", 0, 40);
    c.request_loan(U128(3 * NEAR), U64(60), None);
}

#[test]
#[should_panic(expected = "not past due")]
fn claim_before_due_fails() {
    let mut c = setup();
    let id = activate(&mut c, "ada.testnet", NEAR, 60, 0);
    ctx("anyone.testnet", 0, 69);
    c.claim(id);
}

#[test]
fn default_claims_guarantors_pro_rata() {
    let mut c = setup();
    let id = activate(&mut c, "dayo.testnet", 2 * NEAR, 60, 0);
    ctx("dayo.testnet", NEAR / 2, 30);
    c.repay(id);
    ctx("anyone.testnet", 0, 71);
    let claimed = c.claim(id);
    assert_eq!(claimed.0, 3 * NEAR / 2);
    let l = c.get_loan(id).unwrap();
    assert_eq!(l.status, LoanStatus::Defaulted);
    assert_eq!(l.claimed.0, 3 * NEAR / 2);
    for g in ["bola.testnet", "chidi.testnet"] {
        let m = c.get_member(acc(g)).unwrap().member;
        assert_eq!(m.locked.0, 0);
        // pledged 1, lost 0.75
        assert_eq!(m.savings.0, 3 * NEAR - 3 * NEAR / 4);
        assert_eq!(m.guarantees_claimed, 1);
    }
    let d = c.get_member(acc("dayo.testnet")).unwrap();
    assert_eq!(d.member.loans_defaulted, 1);
    assert_eq!(d.limit.0, 0);
    ctx("anyone.testnet", 0, 80);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.claim(id)));
    assert!(r.is_err(), "double claim must fail");
}

#[test]
fn claim_rounding_is_exact() {
    let mut c = setup();
    // odd amount, uneven pledges
    ctx("ada.testnet", 0, 0);
    let amount = 2 * NEAR - 7;
    let id = c.request_loan(U128(amount), U64(10), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(amount / 3));
    ctx("chidi.testnet", 0, 0);
    c.pledge(id, U128(amount - amount / 3));
    ctx("ada.testnet", NEAR / 10, 0);
    let _ = c.file_proposal(id);
    c.internal_on_filed(id, Some(1));
    c.internal_on_status(id, "Approved");
    ctx("ada.testnet", 333, 5);
    c.repay(id);
    ctx("x.testnet", 0, 11);
    assert_eq!(c.claim(id).0, amount - 333);
    let lost: u128 = ["bola.testnet", "chidi.testnet"]
        .iter()
        .map(|g| c.get_member(acc(g)).unwrap().member.claimed_from_me.0)
        .sum();
    assert_eq!(lost, amount - 333);
}

#[test]
fn rejected_proposal_releases_pledges() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    ctx("chidi.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    ctx("ada.testnet", NEAR / 10, 0);
    let _ = c.file_proposal(id);
    c.internal_on_filed(id, Some(3));
    c.internal_on_status(id, "InProgress");
    assert_eq!(c.get_loan(id).unwrap().status, LoanStatus::Filed);
    c.internal_on_status(id, "Rejected");
    assert_eq!(c.get_loan(id).unwrap().status, LoanStatus::Cancelled);
    assert_eq!(c.get_member(acc("bola.testnet")).unwrap().member.locked.0, 0);
}

#[test]
fn failed_filing_reopens_loan() {
    let mut c = setup();
    ctx("ada.testnet", 0, 0);
    let id = c.request_loan(U128(NEAR), U64(60), None);
    ctx("bola.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    ctx("chidi.testnet", 0, 0);
    c.pledge(id, U128(NEAR / 2));
    ctx("ada.testnet", NEAR / 10, 0);
    let _ = c.file_proposal(id);
    c.internal_on_filed(id, None);
    let l = c.get_loan(id).unwrap();
    assert_eq!(l.status, LoanStatus::Open);
    assert_eq!(l.bond.0, 0);
}

#[test]
#[should_panic(expected = "already have a loan")]
fn one_loan_at_a_time() {
    let mut c = setup();
    activate(&mut c, "ada.testnet", NEAR, 60, 0);
    ctx("ada.testnet", 0, 20);
    c.request_loan(U128(NEAR), U64(60), None);
}

#[test]
#[should_panic(expected = "amount exceeds free savings")]
fn locked_savings_not_withdrawable() {
    let mut c = setup();
    activate(&mut c, "ada.testnet", 2 * NEAR, 60, 0);
    ctx("bola.testnet", 0, 20);
    let _ = c.withdraw(U128(3 * NEAR));
}
