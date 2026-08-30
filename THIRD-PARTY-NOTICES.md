# Third-party notices

ClipVault is [MIT licensed](LICENSE). It also bundles the following third-party source code, whose license requires separate attribution.

## UnRAR source code

Used to list the contents of `.rar` archives (read-only — ClipVault never creates or modifies RAR files), via the [`unrar`](https://crates.io/crates/unrar) crate, which vendors and statically compiles the official UnRAR source into ClipVault's own executable (no separate DLL or installation required).

> The source code of UnRAR utility is freeware. This means:
>
> 1. All copyrights to RAR and the utility UnRAR are exclusively owned by the author - Alexander Roshal.
> 2. UnRAR source code may be used in any software to handle RAR archives without limitations free of charge, but cannot be used to develop RAR (WinRAR) compatible archiver and to re-create RAR compression algorithm, which is proprietary. Distribution of modified UnRAR source code in separate form or as a part of other software is permitted, provided that full text of this paragraph, starting from "UnRAR source code" words, is included in license, or in documentation if license is not available, and in source code comments of resulting package.
> 3. The UnRAR utility may be freely distributed. It is allowed to distribute UnRAR inside of other software packages.
> 4. THE RAR ARCHIVER AND THE UnRAR UTILITY ARE DISTRIBUTED "AS IS". NO WARRANTY OF ANY KIND IS EXPRESSED OR IMPLIED. YOU USE AT YOUR OWN RISK. THE AUTHOR WILL NOT BE LIABLE FOR DATA LOSS, DAMAGES, LOSS OF PROFITS OR ANY OTHER KIND OF LOSS WHILE USING OR MISUSING THIS SOFTWARE.
> 5. Installing and using the UnRAR utility signifies acceptance of these terms and conditions of the license.
> 6. If you don't agree with terms of the license you must remove UnRAR files from your storage devices and cease to use the utility.
>
> — Alexander L. Roshal
