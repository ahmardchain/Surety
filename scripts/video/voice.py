#!/usr/bin/env python3
"""Voiceover with Microsoft Edge neural TTS (free, needs `pip install edge-tts` and access to speech.platform.bing.com).
Writes vo/p1..p7.mp3 and voice.mp3 (paragraphs joined by 0.9 s pauses) next to this file."""
import os, subprocess, time
D = os.path.dirname(os.path.abspath(__file__))
P = [
 "Savings groups can pool money in a DAO. What they can't do safely, is lend it to their own members.",
 "Surety fixes that. A member asks for a loan. Two other members lock part of their savings behind it. That's the guarantee, not collateral.",
 "When the loan is fully covered, Surety files a normal Sputnik DAO transfer. The council approves it once, in Sputnik or Trezu. The money never leaves the DAO's control.",
 "Loan A: the borrower repays. The money goes straight back to the treasury, the guarantors' savings unlock, and everyone's borrowing limit goes up, from one, to one point five NEAR. Access is earned by repaying.",
 "Loan B: the second borrower pays zero point two, and stops. Once it's past due, anyone can call claim.",
 "One transaction. Zero point four NEAR from each guarantor's locked savings moves to the treasury, and it's whole again. No chasing. No bad-debt vote. And the defaulter's limit drops to zero.",
 "Same DAO, two loans, every step on NEAR testnet, with a transaction link. Built on Sputnik DAO and named accounts. The repo and the live board are linked below.",
]
os.makedirs(os.path.join(D, "vo"), exist_ok=True)
proxy = os.environ.get("HTTPS_PROXY")
for i, t in enumerate(P, 1):
    out = os.path.join(D, "vo", f"p{i}.mp3")
    for _ in range(5):
        cmd = ["edge-tts", "--voice", "en-US-AndrewNeural", "--rate=+4%", "--text", t, "--write-media", out] + (["--proxy", proxy] if proxy else [])
        if subprocess.run(cmd, capture_output=True).returncode == 0 and os.path.getsize(out) > 5000: break
        time.sleep(3)
    else:
        raise SystemExit(f"paragraph {i} failed")
args, fc = [], ""
for i in range(1, 8): args += ["-i", os.path.join(D, "vo", f"p{i}.mp3")]
args += ["-f", "lavfi", "-t", "0.9", "-i", "anullsrc=r=24000:cl=mono"]
for i in range(7): fc += f"[{i}:a]" + ("[7:a]" if i < 6 else "")
subprocess.run(["ffmpeg", "-v", "error", "-y", *args, "-filter_complex", fc + "concat=n=13:v=0:a=1,loudnorm=I=-16:TP=-1.5[a]", "-map", "[a]", "-ar", "44100", os.path.join(D, "voice.mp3")], check=True)
print("wrote voice.mp3")
