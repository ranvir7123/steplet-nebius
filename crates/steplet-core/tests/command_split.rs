use proptest::prelude::*;
use steplet_core::command::{NotUnderstood, split};

fn w(words: &[&str]) -> Result<Vec<String>, NotUnderstood> {
    Ok(words.iter().map(|x| x.to_string()).collect())
}

#[test]
fn plain_words_and_spaces() {
    assert_eq!(split("cargo test --release"), w(&["cargo", "test", "--release"]));
    assert_eq!(split("  wc  -l   README "), w(&["wc", "-l", "README"]));
    assert_eq!(split("go test ./..."), w(&["go", "test", "./..."]));
    assert_eq!(split("cargo test --features=a,b"), w(&["cargo", "test", "--features=a,b"]));
}

#[test]
fn quotes_make_one_word() {
    assert_eq!(split(r#"pytest -k "a or b""#), w(&["pytest", "-k", "a or b"]));
    assert_eq!(split("pytest -k 'a or b'"), w(&["pytest", "-k", "a or b"]));
    assert_eq!(split(r#"echo --name='x y'"z""#), w(&["echo", "--name=x yz"]));
    assert_eq!(split(r#"echo "say \"hi\" \\ ok""#), w(&["echo", r#"say "hi" \ ok"#]));
    assert_eq!(split("git commit -m ''"), w(&["git", "commit", "-m", ""]));
    assert_eq!(split("echo '$HOME; | & * \\'"), w(&["echo", "$HOME; | & * \\"]));
}

#[test]
fn shell_symbols_are_not_read() {
    for (line, symbol) in [
        ("cargo test | tail", '|'),
        ("make && make test", '&'),
        ("make; make test", ';'),
        ("sort < list", '<'),
        ("echo hi > out", '>'),
        ("echo $HOME", '$'),
        ("echo `id`", '`'),
        ("ls *.rs", '*'),
        ("ls ?.rs", '?'),
        ("ls [ab].rs", '['),
        ("cat ~/notes", '~'),
        ("echo {a,b}", '{'),
        ("echo #note", '#'),
        ("echo a\\ b", '\\'),
        ("(make)", '('),
        ("echo !1", '!'),
        ("echo café", 'é'),
        ("echo \"$HOME\"", '$'),
        ("echo \"a`b\"", '`'),
        ("echo \"hi!\"", '!'),
        ("echo \"a\\nb\"", '\\'),
    ] {
        assert_eq!(split(line), Err(NotUnderstood::Symbol(symbol)), "{line}");
    }
}

#[test]
fn other_reasons() {
    assert_eq!(split(""), Err(NotUnderstood::Empty));
    assert_eq!(split("   "), Err(NotUnderstood::Empty));
    assert_eq!(split("cargo\ttest"), Err(NotUnderstood::Control));
    assert_eq!(split("make\nmake test"), Err(NotUnderstood::Control));
    assert_eq!(split("echo 'abc"), Err(NotUnderstood::UnclosedQuote));
    assert_eq!(split("echo \"abc"), Err(NotUnderstood::UnclosedQuote));
    assert_eq!(split("echo \"abc\\"), Err(NotUnderstood::UnclosedQuote));
    assert_eq!(split("RUST_LOG=debug cargo test"), Err(NotUnderstood::Assignment));
    assert_eq!(split("echo 'a\u{200b}b'"), Err(NotUnderstood::Symbol('\u{200b}')));
}

proptest! {
    #[test]
    fn single_quoted_words_come_back_unchanged(words in prop::collection::vec("[ -&(-~]{0,12}", 1..6)) {
        let line = words.iter().map(|x| format!("'{x}'")).collect::<Vec<_>>().join(" ");
        if words[0].contains('=') {
            prop_assert_eq!(split(&line), Err(NotUnderstood::Assignment));
        } else {
            prop_assert_eq!(split(&line), Ok(words));
        }
    }
}

#[test]
fn split_gives_the_words_bash_gave_for_every_vector() {
    let mut n = 0;
    for record in include_str!("vectors/commands.txt").lines() {
        let mut fields = record.split('\u{1f}');
        let line = fields.next().unwrap();
        let words: Vec<String> = fields.map(String::from).collect();
        assert_eq!(split(line), Ok(words), "{line:?}");
        n += 1;
    }
    assert_eq!(n, 2000);
}
