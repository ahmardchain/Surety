#!/usr/bin/env python3
"""voice.mp3 + raw scene recording -> surety-demo.mp4 with burned-in captions.
Splits the voiceover into 7 paragraphs at its 6 longest pauses, trims each scene to its paragraph."""
import json, re, subprocess, sys, os
D = os.path.dirname(os.path.abspath(__file__))
VOICE = os.path.join(D, "voice.mp3")
P = [
  "Savings groups can pool money in a DAO. What they can't do safely is lend it to their own members.",
  "Surety fixes that. A member asks for a loan. Two other members lock part of their savings behind it. That's the guarantee, not collateral.",
  "When the loan is fully covered, Surety files a normal Sputnik DAO transfer. The council approves it once, in Sputnik or Trezu. The money never leaves the DAO's control.",
  "Loan A: the borrower repays. The money goes straight back to the treasury, the guarantors' savings unlock, and everyone's borrowing limit goes up, from 1 to 1.5 NEAR. Access is earned by repaying.",
  "Loan B: the second borrower pays 0.2, and stops. Once it's past due, anyone can call claim.",
  "One transaction. 0.4 NEAR from each guarantor's locked savings moves to the treasury, and it's whole again. No chasing. No bad-debt vote. And the defaulter's limit drops to zero.",
  "Same DAO, two loans, every step on NEAR testnet with a transaction link. Built on Sputnik DAO and named accounts. The repo and the live board are linked below.",
]
sh = lambda *a: subprocess.run(a, capture_output=True, text=True)
dur = float(sh("ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", VOICE).stdout)
sd = sh("ffmpeg", "-i", VOICE, "-af", "silencedetect=noise=-35dB:d=0.25", "-f", "null", "-").stderr
starts = [float(x) for x in re.findall(r"silence_start: ([\d.]+)", sd)]
ends = [float(x) for x in re.findall(r"silence_end: ([\d.]+)", sd)]
gaps = [(e - s, s, e) for s, e in zip(starts, ends) if 0.5 < s and e < dur - 0.3]
cuts = sorted(sorted(gaps, reverse=True)[:6], key=lambda g: g[1])
if len(cuts) < 6: sys.exit(f"only {len(cuts)} pauses found: {gaps}")
bounds = [0.0] + [(s + e) / 2 for _, s, e in cuts] + [dur + 1.0]
segs = [(bounds[i], bounds[i + 1]) for i in range(7)]
print("segments:", [round(b - a, 2) for a, b in segs])

m = json.load(open(os.path.join(D, "marks.json")))
start = {x["scene"]: x["t"] for x in m["marks"] if "scene" in x}
endm = {x["end"]: x["t"] - 1.2 for x in m["marks"] if "end" in x}  # margin: never reach the next page load

# video: cut each piece to its own CFR clip (accurate seek), then concat
os.makedirs(os.path.join(D, "pieces"), exist_ok=True)
pieces = []
def cut(src_t, length, pad, name):
    outp = os.path.join(D, "pieces", name + ".mp4")
    vf = f"fps=30,scale=1920:1080:flags=lanczos,tpad=stop_mode=clone:stop_duration={pad:.3f}"
    r = sh("ffmpeg", "-y", "-v", "error", "-ss", f"{src_t:.3f}", "-t", f"{length:.3f}", "-i", m["vid"], "-vf", vf,
           "-an", "-c:v", "libx264", "-preset", "veryfast", "-crf", "16", "-pix_fmt", "yuv420p", outp)
    if r.returncode: sys.exit(r.stderr)
    pieces.append(outp)
for i, (a, b) in enumerate(segs, 1):
    L = b - a
    s0 = start[i]
    gap = [x["t"] for x in m["marks"] if "gap%d" % i in x]
    if len(gap) == 2:
        g0, g1 = gap[0], gap[1] + 1.0
        first = g0 - s0
        cut(s0, first, 0, f"s{i}a")
        rest = L - first
        avail2 = endm[i] - g1
        cut(g1, min(rest, avail2), max(0, rest - avail2), f"s{i}b")
    else:
        avail = endm[i] - s0
        cut(s0, min(L, avail), max(0, L - avail), f"s{i}")
lst = os.path.join(D, "pieces", "list.txt")
open(lst, "w").write("".join(f"file '{p}'\n" for p in pieces))
joined = os.path.join(D, "pieces", "joined.mp4")
r = sh("ffmpeg", "-y", "-v", "error", "-f", "concat", "-safe", "0", "-i", lst, "-c", "copy", joined)
if r.returncode: sys.exit(r.stderr)

# captions: sentences inside each paragraph, timed by character count
def ts(t):
    h, r = divmod(t, 3600); mi, s = divmod(r, 60)
    return f"{int(h)}:{int(mi):02d}:{s:05.2f}"
ev = []
for (a, b), text in zip(segs, P):
    sents = re.split(r"(?<=[.!?])\s+", text)
    sents = [x.strip() for x in sents if x.strip()]
    tot = sum(len(x) for x in sents); t = a + 0.15; span = (min(b, dur) - a - 0.3)
    for x in sents:
        d = span * len(x) / tot
        ev.append(f"Dialogue: 0,{ts(t)},{ts(t + d)},Cap,,0,0,0,,{x}")
        t += d
ev.append(f"Dialogue: 1,{ts(0)},{ts(4.5)},Tag,,0,0,0,,Built at NEAR HACKATHON  ·  #NEARHackathon")
ev.append(f"Dialogue: 1,{ts(segs[-1][0])},{ts(dur + 1)},Tag,,0,0,0,,github.com/ahmardchain/surety  ·  surety.ahmardchain.workers.dev")
ass = f"""[Script Info]
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Cap,Inter,46,&H00FFFFFF,&H00FFFFFF,&H00000000,&H90101012,1,0,0,0,100,100,0,0,3,18,0,2,160,160,64,1
Style: Tag,Inter,34,&H00FFFFFF,&H00FFFFFF,&H00000000,&H0081B910,1,0,0,0,100,100,0,0,3,14,0,8,60,60,40,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
""" + "\n".join(ev) + "\n"
open(os.path.join(D, "captions.ass"), "w").write(ass)

out = os.path.join(D, "surety-demo.mp4")
r = sh("ffmpeg", "-y", "-v", "error", "-i", joined, "-i", VOICE, "-vf", "subtitles=" + os.path.join(D, "captions.ass"),
       "-map", "0:v", "-map", "1:a", "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-pix_fmt", "yuv420p",
       "-c:a", "aac", "-b:a", "160k", "-af", "apad", "-shortest", "-movflags", "+faststart", out)
if r.returncode: sys.exit(r.stderr)
print("wrote", out, sh("ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", out).stdout.strip(), "s")
