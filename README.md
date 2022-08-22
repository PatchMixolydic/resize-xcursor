# `resize-xcursor`
A convenient little command line utility I hacked together to resize [Xcursor files].
This might be useful if you want to use an old X11 cursor theme on a HiDPI display.

Note that this utility currently only supports integer scales larger than 100%.

[Xcursor files]: https://man.archlinux.org/man/extra/libxcursor/Xcursor.3.en#CURSOR_FILES

## Examples
Resize one cursor to 200% scale:
```console
$ resize-xcursor -s 2 my-cursor
```

Resize two cursors to 200% scale, then output the results with new names:
```console
$ resize-xcursor -s 2 my-cursor -o my-cursor-big my-other-cursor -o my-other-cursor-big
```

Resize an entire directory of cursors to 200% scale, ignoring any files that aren't Xcursors:
```console
$ resize-xcursor -s 2 --ignore-unrecognized *
```

## License
This project is licensed under either the [Apache License 2.0] or the [MIT license],
at your option. Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you (as defined in the Apache 2.0 license) shall be licensed under
both licenses, without any additional terms or conditions.

[Apache License 2.0]: ./LICENSE-APACHE
[MIT license]: ./LICENSE-MIT

