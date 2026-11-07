Output ./benchmarks/structaura-benchmark.gif

Set Shell "zsh"
Set FontSize 15
Set Width 1100
Set Height 760
Set Padding 24
Set Theme {
  "name": "Monochrome",
  "black": "#171717",
  "red": "#a0a0a0",
  "green": "#c0c0c0",
  "yellow": "#b0b0b0",
  "blue": "#909090",
  "magenta": "#a8a8a8",
  "cyan": "#b8b8b8",
  "white": "#e5e5e5",
  "brightBlack": "#555555",
  "brightRed": "#b0b0b0",
  "brightGreen": "#d0d0d0",
  "brightYellow": "#c0c0c0",
  "brightBlue": "#b0b0b0",
  "brightMagenta": "#c0c0c0",
  "brightCyan": "#d0d0d0",
  "brightWhite": "#ffffff",
  "background": "#111111",
  "foreground": "#d4d4d4",
  "selection": "#383838",
  "cursor": "#ffffff"
}

Hide
Type "clear"
Enter
Show

Sleep 1s
Type "./scripts/bench.sh 2>&1 | tee benchmark-results.txt"
Enter
Sleep 2s
Sleep 20s
