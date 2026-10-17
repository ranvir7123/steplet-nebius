# Lists every folder the account running this script can write, on every mounted file system except /proc, /sys
# and /dev/pts (phase 4 Task 1, Step 1). Test support only: it changes nothing. Run as the steplet account, through
# `bash -s`; the body is one function, so no program it starts can read the script from standard input.
main() {
    echo "account: $(id -un)"
    find / \( -path /proc -o -path /sys -o -path /dev/pts \) -prune -o -type d -writable -print 2> /dev/null | sort
    echo "end of list"
}
main < /dev/null
