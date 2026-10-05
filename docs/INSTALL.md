# Installing ISC MingoSolve

For teammates. You do not need to be a programmer, and you do not need Rust, Node or Python for any of this.

## 1. Download

Open <https://github.com/isc-fs/iskapps/releases> and pick the newest release whose name starts with
`ISC MingoSolve` (the tag looks like `mingosolve-v0.2.0`). Under **Assets**, download the file for your computer:

| Your computer | File to download |
|---|---|
| Mac (Apple Silicon or Intel) | the `.dmg` (one file works on both) |
| Windows 10 / 11 | the `.exe` ending in `-setup.exe` |
| Linux, any distribution | the `.AppImage` |
| Linux, Debian / Ubuntu | the `.deb` (or the `.AppImage`) |
| Linux, Fedora / openSUSE | the `.rpm` (or the `.AppImage`) |

Ignore the `.sig` files and `latest.json`: they are for the automatic updater.

## 2. Install

### macOS

The app is not notarized by Apple (that costs a yearly fee), so macOS blocks it the first time. This is expected. It
takes one extra step, once.

1. Open the `.dmg` and drag **ISC MingoSolve** into **Applications**.
2. Open it from Applications. macOS says it cannot verify the app; click **Done** (not "Move to Bin").
3. Open **System Settings → Privacy & Security**, scroll down to the **Security** section, and click **Open Anyway**
   next to "ISC MingoSolve was blocked". Enter your password. Confirm with **Open Anyway** once more.

After that it opens like any other app.

Alternative, from the Terminal (paste the line and press Enter), which removes the block in one go:

```bash
xattr -dr com.apple.quarantine "/Applications/ISC MingoSolve.app"
```

### Windows

1. Run the downloaded `-setup.exe`.
2. If a blue **Windows protected your PC** (SmartScreen) window appears, click **More info**, then **Run anyway**. It
   appears because the installer is not signed with a paid certificate.
3. The app needs Microsoft Edge WebView2. It is already part of Windows 11 and of up-to-date Windows 10; if it is
   missing, the installer downloads it (an internet connection is needed during installation).

### Linux

- **AppImage**: make it executable once, then run it. From the folder where you downloaded it:
  ```bash
  chmod +x ISC*MingoSolve*.AppImage
  ./ISC*MingoSolve*.AppImage
  ```
  (Or right-click the file, Properties, "Allow executing file as program".) If it complains about FUSE, install
  `libfuse2` (Ubuntu 22.04+: `sudo apt install libfuse2`).
- **.deb**: `sudo apt install ./ISC*MingoSolve*.deb`
- **.rpm**: `sudo dnf install ./ISC*MingoSolve*.rpm`

## 3. Updates

The app checks for a newer version when it starts and shows a banner with an **Install** button. The button
downloads the update, installs it and restarts the app. You can also check by hand in **Settings → Updates →
Check for updates**.

On Linux, only the **AppImage** updates itself. If you installed the `.deb` or `.rpm`, download the new file from the
releases page and install it again.

### Turning automatic checks off (recommended before a quiz)

Open **Settings → Updates** and untick **Check for updates automatically when the app starts**. The app then never
contacts the update server on its own. Do this the evening before a quiz, after you have the version you want to use,
so nothing changes under you while you are answering. The manual button still works whenever you choose.

## 4. Rulebook search: download the PDF, load it once

The **Rules** view searches the official FS-Rules offline (by rule number such as `T 2.9.2`, or by words such as
`tread depth`), and ⌘K / Ctrl K lists matching rules under the scripts. The rules are copyrighted by Formula Student
Germany, so MingoSolve does not include them: you load the PDF yourself.

1. Download the rules PDF for your year from <https://www.formulastudent.de/rules/> (2027 v1.0 is
   <https://www.formulastudent.de/fileadmin/user_upload/all/2027/rules/FS_Rules_2027_v1.0.pdf>).
2. Open **Rules**, press **Load PDF** on that year's row and pick the file. Reading takes a second or two.
3. Search. The text is saved on your computer (in the app's data folder, one file per year), so you load it only once;
   **Remove** deletes it. Nothing is uploaded and nothing is stored in the repository.

If loading says it found too few rules, the PDF is not the official rulebook (or is a scan without text).

## 5. If the app does not work

On the day of a quiz, do not troubleshoot: use the fallback.

- **Command line (needs the source code and Rust)**: from the repository folder, `cargo run --release` starts the
  interactive solver.
- **Python version (needs [uv](https://docs.astral.sh/uv/))**: from the repository folder,
  `cd legacy/python && uv sync && uv run fsq`.

Both read the same formulas and give the same answers as the app. Report the problem afterwards at
<https://github.com/isc-fs/MingoSolve/issues> with your operating system and what you saw on screen.
