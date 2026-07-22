# semtree demo - render with:  vhs assets/demo.tape   (from the repo root)
Output assets/demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 1200
Set Height 720
Set Padding 24
Set TypingSpeed 45ms
Set WindowBar Colorful

# Setup (hidden): put the release binary on PATH, quiet logs, clean prompt.
Hide
Type `export PATH="$PWD/target/release:$PATH"; export RUST_LOG=off; export PS1='$ '`
Enter
Type "rm -rf .semtree; clear"
Enter
Show

Sleep 700ms
Type "semtree index ./crates/semtree-rag/src"
Sleep 400ms
Enter
Sleep 5s

Type `semtree search "build an LLM context window from search hits" --mode semantic -t 2`
Sleep 400ms
Enter
Sleep 3200ms

Type `semtree search "run a hybrid search fusing semantic and keyword results" --mode semantic -t 2`
Sleep 400ms
Enter
Sleep 3200ms

Type "semtree stats"
Sleep 400ms
Enter
Sleep 3500ms
