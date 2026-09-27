//! Record real public CLI runs and render edited, explicitly labelled GIF replays.
//! Build with Rust 1.85.0. FFmpeg and system fonts are rendering tools, not vendored.
#![forbid(unsafe_code)]
use std::{env, fs, io::Write, path::{Path, PathBuf}, process::{Command, Stdio}};
type R<T> = Result<T, Box<dyn std::error::Error>>;
const FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf";
const ITEMS: [(&str, &str); 6] = [
    ("01-evidence", "Exact quotes. Verified restart."),
    ("02-stale", "Changed source. Old citation refused."),
    ("03-backup", "Backup. Inspect. Restore to a new path."),
    ("04-reproduce", "One command. Inspect every result."),
    ("05-integrity", "One changed byte. Trusted pin rejects it."),
    ("06-compare", "GEL and grep. Compare answers first."),
];
fn capture(program: &str, args: &[&str], cwd: &Path, input: &str, code: i32) -> R<(String, String)> {
    let mut child = Command::new(program).args(args).current_dir(cwd)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("missing stdin")?.write_all(input.as_bytes())?;
    let out = child.wait_with_output()?;
    if out.stdout.len() + out.stderr.len() > 16 * 1024 * 1024 { return Err("output limit".into()); }
    let stdout = String::from_utf8(out.stdout)?;
    let stderr = String::from_utf8(out.stderr)?;
    if out.status.code() != Some(code) {
        return Err(format!("{program}: expected exit {code}, got {}\n{stdout}\n{stderr}", out.status).into());
    }
    Ok((stdout, stderr))
}
fn checked(program: &str, args: &[&str], cwd: &Path, input: &str, code: i32) -> R<String> {
    Ok(capture(program, args, cwd, input, code)?.0)
}
fn hash(path: &Path) -> R<String> {
    let o = checked("sha256sum", &[path.to_str().ok_or("non UTF-8 path")?], Path::new("."), "", 0)?;
    let h = o.split_whitespace().next().ok_or("missing hash")?;
    if h.len() != 64 || !h.bytes().all(|c| c.is_ascii_hexdigit()) { return Err("invalid hash".into()); }
    Ok(h.to_owned())
}
fn contains(s: &str, needle: &str) -> R<()> {
    if s.contains(needle) { Ok(()) } else { Err(format!("missing observed marker: {needle}").into()) }
}
fn line(s: &str, needle: &str) -> R<String> {
    Ok(s.lines().find(|l| l.contains(needle)).ok_or_else(|| format!("missing line {needle}"))?
        .replace("gel> ", ""))
}
fn excerpt(s: &str, needle: &str) -> R<String> {
    let l = line(s, needle)?;
    Ok(l[l.find(needle).ok_or("missing excerpt")?..].to_owned())
}
fn wrap(text: &str) -> String {
    let mut out = String::new();
    for l in text.lines() {
        let l = l.replace('\t', "  ");
        let chars: Vec<_> = l.chars().filter(|c| !c.is_control()).collect();
        if chars.is_empty() { out.push('\n'); }
        for ch in chars.chunks(72) { out.extend(ch); out.push('\n'); }
    }
    out
}
fn picture(base: &str, alt: &str) -> String {
    format!("<picture>\n  <source media=\"(prefers-color-scheme: dark)\" srcset=\"{base}-dark.gif\">\n  <img alt=\"{alt}\" src=\"{base}-light.gif\" width=\"1000\">\n</picture>\n")
}
fn render(dir: &Path, temp: &Path, slug: &str, title: &str, source: &str, cards: &[String]) -> R<Vec<(String,String)>> {
    if cards.len() != 3 { return Err("three cards required".into()); }
    let mut pins = Vec::new();
    for (theme, bg, ink) in [("light", "ffffff", "0b2447"), ("dark", "0d1117", "e6edf3")] {
        let accent = if theme == "light" { "0969da" } else { "4493f8" };
        let frames = temp.join(format!("{slug}-{theme}")); fs::create_dir(&frames)?;
        fs::write(frames.join("title.txt"), title)?;
        fs::write(frames.join("foot.txt"), format!("EDITED LOG REPLAY / NOT WALL TIME / source {}", &source[..8]))?;
        for (i, card) in cards.iter().enumerate() {
            let body = wrap(card);
            if body.lines().count() > 9 { return Err(format!("card exceeds nine lines: {slug}\n{body}").into()); }
            fs::write(frames.join("body.txt"), &body)?;
            fs::write(frames.join("step.txt"), format!("GEL RAM  /  EVIDENCE LAB                             {} / 3", i + 1))?;
            let filt = format!("drawbox=x=18:y=100:w=964:h=306:color=0x071b2b:t=fill,drawbox=x=18:y=100:w={}:h=4:color=0x{accent}:t=fill,drawtext=fontfile={FONT}:textfile=title.txt:expansion=none:fontcolor=0x{ink}:fontsize=28:x=26:y=22,drawtext=fontfile={FONT}:textfile=step.txt:expansion=none:fontcolor=0x{accent}:fontsize=17:x=26:y=67,drawtext=fontfile={FONT}:textfile=body.txt:expansion=none:fontcolor=0xe6edf3:fontsize=20:line_spacing=7:x=36:y=120,drawtext=fontfile={FONT}:textfile=foot.txt:expansion=none:fontcolor=0x{ink}:fontsize=15:x=26:y=424", 964*(i+1)/cards.len());
            let frame = format!("frame-{i:02}.png");
            checked("ffmpeg", &["-nostdin","-y","-v","error","-f","lavfi","-i", &format!("color=c=0x{bg}:s=1000x460"),"-vf", &filt,"-frames:v","1","-threads","1",&frame], &frames, "", 0)?;
        }
        let gif = dir.join(format!("{slug}-{theme}.gif"));
        checked("ffmpeg", &["-nostdin","-y","-v","error","-framerate","1/4","-i","frame-%02d.png","-filter_complex","split[a][b];[a]palettegen=max_colors=64:stats_mode=diff[p];[b][p]paletteuse=dither=none","-threads","1","-loop","0",gif.to_str().ok_or("gif path")?], &frames, "", 0)?;
        // Decode every generated frame; do not treat file existence as validation.
        checked("ffmpeg", &["-nostdin","-v","error","-i",gif.to_str().ok_or("gif path")?,"-f","null","-"], &frames, "", 0)?;
        let probe = checked("ffprobe", &["-v","error","-count_frames","-select_streams","v:0","-show_entries","stream=width,height,nb_read_frames:format=duration","-of","default=noprint_wrappers=1",gif.to_str().ok_or("gif path")?], &frames, "", 0)?;
        for marker in ["width=1000", "height=460", "nb_read_frames=3", "duration=12."] { contains(&probe, marker)?; }
        if fs::metadata(&gif)?.len() > 512*1024 { return Err("GIF exceeds 512 KiB budget".into()); }
        let poster = dir.join(format!("{slug}-{theme}.png"));
        fs::copy(frames.join("frame-00.png"), &poster)?;
        for p in [gif, poster] { pins.push((format!("media/gifs/{}",p.file_name().ok_or("filename")?.to_str().ok_or("filename")?), hash(&p)?)); }
        fs::write(dir.join(format!("{slug}-{theme}-decode.txt")), probe)?;
    }
    Ok(pins)
}
fn main() -> R<()> {
    let args: Vec<_> = env::args().collect();
    if args.len()!=2 { return Err("usage: readme_media NEW_TEMP_DIR_OUTSIDE_CHECKOUT".into()); }
    let root=env::current_dir()?.canonicalize()?;
    let temp=PathBuf::from(&args[1]); fs::create_dir(&temp)?; let temp=temp.canonicalize()?;
    if temp.starts_with(&root) { return Err("scratch directory must be outside checkout".into()); }
    let source=checked("git", &["rev-parse","HEAD"], &root,"",0)?.trim().to_owned();
    if !checked("git", &["status","--porcelain"], &root,"",0)?.is_empty() { return Err("clean source checkout required".into()); }
    let source_pin=hash(&root.join("SOURCE-SHA256SUMS.txt"))?;
    let ev=root.join("target/release/gel-evidence"); let ev=ev.to_str().ok_or("binary path")?;
    let backup=root.join("target/release/gel-backup"); let backup=backup.to_str().ok_or("binary path")?;
    let xtask=root.join("target/debug/xtask"); let xtask=xtask.to_str().ok_or("binary path")?;
    fs::write(temp.join("original.txt"), "The sample pressure is 2 bar.\n")?;
    fs::write(temp.join("revised.txt"), "The sample pressure is 3 bar.\n")?;
    let first_in="add original.txt\nfind sample pressure\nproof 1\nsave snapshot.gelset\nexit\n";
    let first=checked(ev,&[],&temp,first_in,0)?;
    for m in ["ADDED id=1", "FIND=HIT", "CITATION=PASS", "SAVED revision="] { contains(&first,m)?; }
    let pin=excerpt(&first,"BUNDLE_SHA256=")?.trim_start_matches("BUNDLE_SHA256=").split(';').next().ok_or("pin")?.to_owned();
    if pin != hash(&temp.join("snapshot.gelset"))? { return Err("saved pin differs from independent sha256sum".into()); }
    let reopen_in=format!("load {pin} snapshot.gelset\nfind sample pressure\nproof 1\nexit\n");
    let reopen=checked(ev,&[],&temp,&reopen_in,0)?;
    contains(&reopen,"REOPEN=PASS")?; contains(&reopen,"CITATION=PASS")?;
    let q=excerpt(&first,"QUOTE ")?;
    if excerpt(&reopen,"QUOTE ")? != q { return Err("reopened quote differs".into()); }
    let stale_in="add original.txt\nfind sample pressure\nreplace 1 revised.txt\nproof 1\nexit\n";
    let (stale, stale_err)=capture(ev,&["--batch"],&temp,stale_in,2)?;
    contains(&stale,"\"previous_citations_invalidated\":true")?; contains(&stale,"\"error\":\"NO_CURRENT_RESULT\"")?;
    let b1=checked(backup,&["create",&pin,"snapshot.gelset","backup"],&temp,"",0)?;
    let b2=checked(backup,&["inspect",&pin,"backup"],&temp,"",0)?;
    let b3=checked(backup,&["restore",&pin,"backup","restored.gelset"],&temp,"",0)?;
    contains(&b3,"RESTORE=PASS")?;
    if fs::read(temp.join("snapshot.gelset"))? != fs::read(temp.join("restored.gelset"))? { return Err("backup bytes differ".into()); }
    let mut corrupt=fs::read(temp.join("snapshot.gelset"))?;
    let last=corrupt.last_mut().ok_or("empty snapshot")?; *last ^= 1;
    fs::write(temp.join("corrupted.gelset"), &corrupt)?;
    let corrupt_in=format!("load {pin} corrupted.gelset\nexit\n");
    let (refusal, refusal_err)=capture(ev,&["--batch"],&temp,&corrupt_in,2)?;
    contains(&refusal,"\"status\":\"ERROR\"")?;
    contains(&refusal,"\"error\":\"COLLECTION_INTEGRITY\"")?;
    println!("MEDIA_RECORDING=CLI_CASES_PASSED starting strict reproduction");
    // This is the real reproduction runner, not a simulated stream of PASS labels.
    checked("unshare",&["--user","--net","--",xtask,"reproduce","reproduction","--require-isolation","--strict"],&temp,"",0)?;
    let rep_dir=temp.join("reproduction");
    let rep=fs::read_to_string(rep_dir.join("REPRODUCTION.txt"))?;
    contains(&rep,"REPRODUCTION=PASS pass=3 fail=0 skipped=0 not_run=0 isolation=VERIFIED")?;
    let answers=fs::read_to_string(rep_dir.join("bench/answers.tsv"))
        .map_err(|e| format!("read benchmark answers.tsv: {e}"))?;
    let all=[
        format!("PROGRAM=gel-evidence\nPROCESS=1\nSTDIN\n{first_in}STDOUT\n{first}\nPROCESS=2\nSTDIN\n{reopen_in}STDOUT\n{reopen}\nRECORDER_CHECK=quote_before_equals_quote_after\n"),
        format!("PROGRAM=gel-evidence --batch\nSTDIN\n{stale_in}STDOUT\n{stale}\nSTDERR\n{stale_err}\nEXPECTED_AND_OBSERVED_EXIT=2\n"),
        format!("PROGRAM=gel-backup\ncreate {pin} snapshot.gelset backup\n{b1}\ninspect {pin} backup\n{b2}\nrestore {pin} backup restored.gelset\n{b3}\nRECORDER_CHECK=restored_bytes_equal_snapshot_bytes\n"),
        rep,
        format!("OPERATION=copy snapshot and XOR its last byte with 1; original untouched\nPROGRAM=gel-evidence --batch\nSTDIN\n{corrupt_in}STDOUT\n{refusal}\nSTDERR\n{refusal_err}\nEXPECTED_AND_OBSERVED_EXIT=2\n"),
        format!("Recorded answer agreement, not a speed claim.\n{}\n", answers),
    ];
    let rep_lines=all[3].lines().filter(|s| s.starts_with("step=") || s.starts_with("network_isolation=")).collect::<Vec<_>>().join("\n");
    let refusal_line=line(&refusal,"\"status\":\"ERROR\"")?;
    let rows=answers.lines().take(3).collect::<Vec<_>>().join("\n");
    let cards=vec![
        vec![format!("$ gel-evidence\nadd original.txt\nfind sample pressure\n\n{}\n{q}",excerpt(&first,"ADDED")?),format!("proof 1\nsave snapshot.gelset\n\n{}\n\nBUNDLE_SHA256=\n{pin}", "CITATION=PASS"),format!("New OS process: gel-evidence\nload SAVED_SHA256 snapshot.gelset\nfind sample pressure\n\n{}\n{q}\n\nRecorder verified: quote bytes unchanged.",excerpt(&reopen,"REOPEN=PASS")?)],
        vec![format!("$ gel-evidence --batch\nadd original.txt\nfind sample pressure\n\n{q}"),"replace 1 revised.txt\n\nObserved JSON field:\n\"previous_citations_invalidated\":true".into(),"proof 1\n\nObserved JSON field:\n\"error\":\"NO_CURRENT_RESULT\"\n\nObserved process exit: 2".into()],
        vec![format!("$ gel-backup create SAVED_SHA256 snapshot.gelset backup\n\n{b1}"),format!("$ gel-backup inspect SAVED_SHA256 backup\n\n{b2}"),format!("$ gel-backup restore SAVED_SHA256 backup restored.gelset\n\n{b3}\nRecorder verified: restored bytes equal original.")],
        vec!["$ xtask reproduce reproduction\n    --require-isolation --strict\n\nExecuted inside: unshare --user --net\nThis clip replays the completed report.".into(),rep_lines,format!("Observed final report:\n\n{}\n\nHosted Linux run; not an independent second host.",line(&all[3],"REPRODUCTION=PASS")?)],
        vec!["Copy the saved snapshot.\nFlip its final byte with XOR 1.\n\nThe original remains untouched.\nThe trusted SHA-256 is unchanged.".into(),format!("$ gel-evidence --batch\nload SAVED_SHA256 corrupted.gelset\n\nObserved JSON result:\n{refusal_line}"),"Observed process exit: 2\n\nPin mismatch is not structural-parser coverage.\nThe finite mutation campaign is linked separately.\nA hash checks bytes, not the truth of a source.".into()],
        vec!["GEL and grep: answer agreement\n\nRecorded by xtask reproduce.\nGEL normalizes Unicode and tokenizes phrases.\ngrep uses different matching rules.".into(),format!("Observed answer-table excerpt:\n\n{rows}"),"A shared corpus is not identical semantics.\n\nKeep both SAME and DIFFERENT rows.\nUse the complete answer table linked below.\nThis animation makes no speed claim.".into()],
    ];
    println!("MEDIA_RECORDING=COMPLETE rendering six scenarios");
    let dir=root.join("media/gifs"); fs::create_dir(&dir)?;
    fs::copy(temp.join("original.txt"),dir.join("source-original.txt"))?;
    fs::copy(temp.join("revised.txt"),dir.join("source-revised.txt"))?;
    let mut pins=Vec::new();
    let mut manifest=format!("FORMAT=GEL_README_MEDIA_1\nsource_commit={source}\nsource_manifest_sha256={source_pin}\nrecording=real_public_cli_processes\npresentation=edited_log_replay_not_screen_capture_or_wall_time\nfixture=synthetic_pressure_text\nnew_measurement_claim=none\nplatform=linux_ci\nhuman_visual_acceptance=OPEN\n");
    for tool in ["rustc","ffmpeg"] { manifest.push_str(&checked(tool,&[if tool=="rustc" {"--version"} else {"-version"}],&temp,"",0)?.lines().next().unwrap_or("").to_owned()); manifest.push('\n'); }
    for (i,(slug,title)) in ITEMS.iter().enumerate() {
        let log=all[i].replace(root.to_str().ok_or("root")?,"$CHECKOUT").replace(temp.to_str().ok_or("temp")?,"$RUN_DIR");
        fs::write(dir.join(format!("{slug}.txt")),format!("source_commit={source}\nSANITIZATION=absolute checkout and scratch prefixes replaced; stdout retained\n\n{log}"))?;
        println!("MEDIA_RENDER={slug}");
        pins.extend(render(&dir,&temp,slug,title,&source,&cards[i])?);
    }
    for (name,h) in &pins { manifest.push_str(&format!("sha256 {h}  {name}\n")); }
    manifest.push_str("MEDIA_DECODE=PASS all 12 GIFs, 3 frames each, 1000x460, 12 seconds\n");
    fs::write(dir.join("MANIFEST.txt"),&manifest)?;
    let mut gallery=String::from("# Six public workflow replays\n\nThese are edited replays of real CLI logs, not original screen captures.\nThe four-second cards are editorial pacing, never measured latency.\n[Source run and asset hashes](MANIFEST.txt) · [Static view](STATIC.md) · [Media rights](../RIGHTS.md)\n\nNo private application or invented benchmark is shown. Placeholders such as SAVED_SHA256\nabbreviate command display only; the exact executed arguments are in each transcript.\nCommands shown after gel-evidence are its stdin, not unsupported command-line subcommands.\nExpected failure cases have checked nonzero exits. Full human visual acceptance remains open.\n\n");
    let mut static_view=String::from("# Static view without animated images\n\nPosters and complete text transcripts of each edited replay.\n[Provenance](MANIFEST.txt) · [Media rights](../RIGHTS.md)\n\n");
    let mut section=String::from("## See it in action\n\n**Actual public-tool runs, presented as six edited log replays.**\n[Static view](media/gifs/STATIC.md) · [Full gallery and transcripts](media/gifs/README.md) · [Source and hashes](media/gifs/MANIFEST.txt)\n\nEach animation lasts 12 seconds. Card pacing is editorial, not execution time.\nLight and dark variants match the README theme; these are not product UI screenshots.\n\n");
    for (i,(slug,title)) in ITEMS.iter().enumerate() {
        gallery.push_str(&format!("## {}. {title}\n\n{}\n[Complete transcript]({slug}.txt) · [Full-size animation]({slug}-light.gif)\n\n",i+1,picture(slug,title)));
        static_view.push_str(&format!("## {}. {title}\n\n<picture>\n<source media=\"(prefers-color-scheme: dark)\" srcset=\"{slug}-dark.png\">\n<img alt=\"{title}\" src=\"{slug}-light.png\" width=\"1000\">\n</picture>\n\n[Complete transcript]({slug}.txt) · [Full-size animation]({slug}-light.gif)\n\n",i+1));
        if i==0 { section.push_str(&format!("{}\n[Read the full source/restart transcript](media/gifs/{slug}.txt)\n\n",picture(&format!("media/gifs/{slug}"),title))); }
        else { section.push_str(&format!("<details>\n<summary><strong>{}. {title}</strong></summary>\n\n{}\n[Complete transcript](media/gifs/{slug}.txt) · [Full-size animation](media/gifs/{slug}-light.gif)\n\n</details>\n\n",i+1,picture(&format!("media/gifs/{slug}"),title))); }
    }
    section.push_str("The earlier [70-second Evidence Lab film](media/GEL-EVIDENCE-LAB-EN.mp4),\nits [original process logs](media/EVIDENCE-LAB-GUIDE.md) and\n[open human review](docs/MEDIA-DECODE-REVIEW.md) remain separate historical material.\nThe new replays do not close that review.\n\n");
    fs::write(dir.join("README.md"),gallery)?; fs::write(dir.join("STATIC.md"),static_view)?;
    let readme=fs::read_to_string(root.join("README.md"))?;
    let start=readme.find("## See it in action\n").ok_or("README section start")?;
    let end=readme.find("### What you can inspect\n").ok_or("README section end")?;
    if end<=start { return Err("section order".into()); }
    fs::write(root.join("README.md"),format!("{}{}{}",&readme[..start],section,&readme[end..]))?;
    let index=root.join("media/INDEX.md");
    let old=fs::read_to_string(&index)?;
    fs::write(index,format!("# Animated workflow gallery\n\n[Six edited public CLI replays](gifs/README.md) · [Static accessible view](gifs/STATIC.md) · [Provenance](gifs/MANIFEST.txt)\n\nLight/dark cards are derived from newly executed public CLI logs. They are not\nnew film recordings or latency measurements. The original films below are unchanged.\n\n{old}"))?;
    // Add exact pins to the existing allow-list, never a blanket GIF extension exception.
    let main=root.join("xtask/src/main.rs"); let text=fs::read_to_string(&main)?;
    let marker="const REVIEWED_ASSETS: &[(&str, &str)] = &[\n";
    if text.matches(marker).count()!=1 { return Err("asset-list marker ambiguous".into()); }
    let entries=pins.iter().map(|(p,h)| format!("    (\n        \"{p}\",\n        \"{h}\",\n    ),\n")).collect::<String>();
    fs::write(main,text.replacen(marker,&format!("{marker}{entries}"),1))?;
    println!("READMEMEDIA_GENERATED=PASS source={source} assets={} logs=6",pins.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test] fn wraps_without_losing_unicode() {
        let input = "x".repeat(73) + "ż";
        let output = super::wrap(&input);
        assert_eq!(output.lines().collect::<String>(), input);
        assert!(output.lines().all(|l| l.chars().count() <= 72));
    }
    #[test] fn missing_observation_is_not_a_pass() {
        assert!(super::contains("REFUSED", "CITATION=PASS").is_err());
        assert!(super::line("", "REOPEN=PASS").is_err());
    }
    #[test] fn prompt_removal_does_not_change_quote() {
        assert_eq!(super::excerpt("gel> QUOTE 2 bar", "QUOTE ").unwrap(), "QUOTE 2 bar");
    }
    #[test] fn six_distinct_scenarios() {
        let slugs: std::collections::BTreeSet<_> = super::ITEMS.iter().map(|(s,_)| s).collect();
        assert_eq!(slugs.len(),6);
    }
}
