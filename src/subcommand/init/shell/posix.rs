use super::{HookConfig, ShellConfig};

use anyhow::{anyhow, Result};
use uuid::Uuid;

use std::borrow::Cow;

pub const CONFIG: ShellConfig = ShellConfig {
    z,
    alias,
    hook: HookConfig {
        prompt: HOOK_PROMPT,
        pwd: hook_pwd,
    },
};

fn z(z_cmd: &str) -> String {
    format!(
        r#"
_z_cd() {{
    cd "$@" || return "$?"

    if [ "$_ZO_ECHO" = "1" ]; then
        echo "$PWD"
    fi
}}

{}() {{
    if [ "$#" -eq 0 ]; then
        _z_cd ~ || return "$?"
    elif [ "$#" -eq 1 ] && [ "$1" = '-' ]; then
        if [ -n "$OLDPWD" ]; then
            _z_cd "$OLDPWD" || return "$?"
        else
            echo 'zoxide: $OLDPWD is not set'
            return 1
        fi
    else
        result="$(zoxide query "$@")" || return "$?"
        if [ -d "$result" ]; then
            _z_cd "$result" || return "$?"
        elif [ -n "$result" ]; then
            echo "$result"
        fi
    fi
}}
"#,
        z_cmd
    )
}

fn alias(z_cmd: &str) -> String {
    format!(
        r#"
alias zi='{} -i'

alias za='zoxide add'

alias zq='zoxide query'
alias zqi='zoxide query -i'

alias zr='zoxide remove'
alias zri='zoxide remove -i'
"#,
        z_cmd
    )
}

const HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add
}

case "$PS1" in
    *\$\(_zoxide_hook\)*) ;;
    *) PS1="\$(_zoxide_hook)${PS1}" ;;
esac
"#;

fn hook_pwd() -> Result<Cow<'static, str>> {
    let mut tmp_path = std::env::temp_dir();
    tmp_path.push("zoxide");

    let tmp_path_str = tmp_path
        .to_str()
        .ok_or_else(|| anyhow!("invalid Unicode in zoxide tmp path"))?;

    let pwd_path = tmp_path.join(format!("pwd-{}", Uuid::new_v4()));

    let pwd_path_str = pwd_path
        .to_str()
        .ok_or_else(|| anyhow!("invalid Unicode in zoxide pwd path"))?;

    let hook_pwd = format!(
        r#"
# PWD hooks in POSIX use a temporary file, located at `$_ZO_PWD_PATH`, to track
# changes in the current directory. These files are removed upon restart,
# but they should ideally also be cleaned up once the shell exits using traps.
#
# This can be done as follows:
#
# trap '_zoxide_cleanup' EXIT HUP KILL TERM
# trap '_zoxide_cleanup; trap - INT; kill -s INT "$$"' INT
# trap '_zoxide_cleanup; trap - QUIT; kill -s QUIT "$$"' QUIT
#
# By default, traps are not set up because they override all previous traps.
# It is therefore up to the user to add traps to their shell configuration.

_ZO_TMP_PATH={}
_ZO_PWD_PATH={}

_zoxide_cleanup() {{
    rm -f "$_ZO_PWD_PATH"
}}

_zoxide_setpwd() {{
    mkdir -p "$_ZO_TMP_PATH"
    echo "$PWD" > "$_ZO_PWD_PATH"
}}

_zoxide_setpwd

_zoxide_hook() {{
    _ZO_OLDPWD="$(cat "$_ZO_PWD_PATH")"
    if [ -z "$_ZO_OLDPWD" ] || [ "$_ZO_OLDPWD" != "$PWD" ]; then
        _zoxide_setpwd && zoxide add > /dev/null
    fi
}}

case "$PS1" in
    *\$\(_zoxide_hook\)*) ;;
    *) PS1="\$(_zoxide_hook)${{PS1}}" ;;
esac"#,
        quote(tmp_path_str),
        quote(pwd_path_str),
    );

    Ok(Cow::Owned(hook_pwd))
}

fn quote(string: &str) -> String {
    let mut quoted = String::with_capacity(string.len() + 2);

    quoted.push('\'');
    for ch in string.chars() {
        match ch {
            '\\' => quoted.push_str(r"\\"),
            '\'' => quoted.push_str(r"'\''"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('\'');

    quoted
}
