## cmsg
Commit message.

You constantly forget what exactly you've worked on in a single commit?
If that sounds like you, then this might just be your savior.

`cmsg` gives you simple yet powerful way to track you work right then and there.
You simply use your language's comment feature and a little `.cmsg` marker and you're done.

### Setup

1. Install the toolchain
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - `rustup toolchain install nightly`
2. Clone the repo
3. `cd cmsg && cargo build --profile native`
4. `cp -f target/native/cmsg ~/.local/bin/`
5. Try `cmsg -h` to verify that everything worked.

### Usage

```rust
fn main() {
    // .cmsg added a print statement to greet the user.
    println!("Hello, world!");
}
```

```sql
-- .cmsg oops i did it again.
DROP DATABASE main;
```

```js
// .cmsg Removed all the buggy j***s***** code.
console.log("finally some peace and quiet in here");
```

Now, when you're done making your changes, and are ready to write your beautiful commit message:

`$ cmsg commit`

which will print out all occurences of `.cmsg` markers and remove them from your code.

Note markers are removed by line, or rather lines with markers are removed.
This means markers such as this:

```php
$requestMethod = $_SERVER["REQUEST_METHOD"] // .cmsg added request-method-based filtering

if (...)
```

Would remove the entire line, and break your code.
That's why you should probably put the comment above!

```php
// .cmsg added request-method-based filtering
$requestMethod = $_SERVER["REQUEST_METHOD"]

if (...)
```

This should work just fine.

Yes, it does create a backup. No, it will not modify your code if creation of the backup fails.

### The CLI

For what it is, it's fairly extensive, even allowing you specify different
formatting options such as vim-compatible output, and json for scripting.

For futher information, run `cmsg -h` and/or `cmsg <sub-command> -h`

### Internals

Internally, it uses the [`ignore`](https://crates.io/crates/ignore) crate
to walk directories in parallel, and does most of the heavy work, such as scanning
the files for `.cmsg` markers in parallel using a custom thread-per-core "thread pool"
(It's just a few channels).
`ignore` also respects ignore files (such as `.gitignore`, `.ignore`, ...)
and hidden files.
This behavior can be turned off via the `-I` and `-H` flags respectively.
`cmsg -IH` will therefore scan the whole directory tree starting from `.`
or `<path>` if `-d <path>` is used.

For metadata storage, `cmsg` uses [`rusqlite`](https://crates.io/crates/rusqlite) which
allows for simple, synchronous interaction with a sqlite DB. (It's pronounced "squeal", btw.)
This database is located at `~/.local/share/cmsg/data.db` or `%USERPROFILE%\AppData\Local\cmsg\data.db`

Committing will store a backup of all files containing `.cmsg` markers in a data directory.
This directory is resolved dynamically using a local-first global-fallback policy.
I plan on making this configurable, but for now, it works like this:

1. If `.`, or any parent directory has a `.git` directory, backups will land inside `.git/cmsg`.
2. If there is no project-local `.git`, it will fall back to `~/.local/share/cmsg/data` or `%USERPROFILE%\AppData\Local\cmsg\data`.
3. (Future only) If the policy is set to local-only, the commit will fail due to not being able to find a suitable data directory.

Did I mention `cmsg` runs on both Linux and Windows?
