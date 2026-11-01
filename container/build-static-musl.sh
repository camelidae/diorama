#!/bin/sh
# Build static musl release binaries (diorama with oci-cri, dioramactl, diorama-addon)
# using pinned Zig 0.17.0 as the C/C++ compiler, linker, and archiver.
set -eu

target=${1:?usage: build-static-musl.sh <x86_64-unknown-linux-musl|aarch64-unknown-linux-musl>}

here=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
root=$(CDPATH= cd -- "$here/.." && pwd)

case "$target" in
    x86_64-unknown-linux-musl)
        arch=x86_64
        zig_target=x86_64-linux-musl
        zig_linux_inc=x86-linux-any
        ;;
    aarch64-unknown-linux-musl)
        arch=aarch64
        zig_target=aarch64-linux-musl
        zig_linux_inc=aarch64-linux-any
        ;;
    *)
        printf 'unsupported musl target: %s\n' "$target" >&2
        exit 1
        ;;
esac

work_root=${RUNNER_TEMP:-$root/target}
mkdir -p "$work_root"

if [ "$(uname -s)" = "Linux" ]; then
    case "$(uname -m)" in
        x86_64)
            host_arch=x86_64
            zig_sha256=1cbe9df9f27e6b78d14ccbca43b6703a404ef79ef1c463de901d7f088d4e2026
            ;;
        aarch64|arm64)
            host_arch=aarch64
            zig_sha256=9e8d11661d4ae3bd57702a3832781e23ad151dde5798e16a5ccd503f65234ff8
            ;;
        *)
            printf 'unsupported Linux host architecture: %s\n' "$(uname -m)" >&2
            exit 1
            ;;
    esac
    zig_dir="$work_root/zig-0.17.0-$host_arch"
    if [ ! -x "$zig_dir/zig" ]; then
        mkdir -p "$zig_dir"
        curl --fail --location --proto '=https' --tlsv1.2 \
            "https://ziglang.org/download/0.17.0/zig-${host_arch}-linux-0.17.0.tar.xz" \
            --output "$zig_dir/zig.tar.xz"
        printf '%s  %s\n' "$zig_sha256" "$zig_dir/zig.tar.xz" | sha256sum --check --strict
        tar -xJf "$zig_dir/zig.tar.xz" -C "$zig_dir" --strip-components=1
        rm -f "$zig_dir/zig.tar.xz"
    fi
    export PATH="$zig_dir:$PATH"
    zig_lib="$zig_dir/lib"
else
    export PATH="/opt/homebrew/bin:${HOME}/.cargo/bin:${PATH}"
    if ! command -v zig >/dev/null 2>&1; then
        printf 'zig is required on PATH for non-Linux hosts\n' >&2
        exit 1
    fi
    if [ -d "/opt/homebrew/lib/zig" ]; then
        zig_lib="/opt/homebrew/lib/zig"
    else
        zig_lib=$(zig env | sed -n 's/.*lib_dir[":= ]*"\([^"]*\)".*/\1/p' | head -n 1)
    fi
fi

wrap_dir="$work_root/zig-wrappers-$arch"
mkdir -p "$wrap_dir"

cat > "$wrap_dir/zig-cc.sh" <<EOF
#!/bin/sh
args=""
for arg in "\$@"; do
    case "\$arg" in
        --target=*|-Wl,--fix-cortex-a53-843419) ;;
        -Wl,/*.a|-Wl,/*.o)
            stripped="\${arg#-Wl,}"
            args="\$args \"\$stripped\""
            ;;
        *) args="\$args \"\$arg\"" ;;
    esac
done
eval "exec zig cc -target $zig_target \$args"
EOF

cat > "$wrap_dir/zig-cxx.sh" <<EOF
#!/bin/sh
args=""
for arg in "\$@"; do
    case "\$arg" in
        --target=*|-Wl,--fix-cortex-a53-843419) ;;
        -Wl,/*.a|-Wl,/*.o)
            stripped="\${arg#-Wl,}"
            args="\$args \"\$stripped\""
            ;;
        *) args="\$args \"\$arg\"" ;;
    esac
done
eval "exec zig c++ -target $zig_target \$args"
EOF

cat > "$wrap_dir/zig-ar.sh" <<'EOF'
#!/bin/sh
exec zig ar "$@"
EOF

cat > "$wrap_dir/zig-ranlib.sh" <<'EOF'
#!/bin/sh
exec zig ranlib "$@"
EOF

cat > "$wrap_dir/go-wrapper.sh" <<EOF
#!/bin/sh
export CC="$wrap_dir/zig-cc.sh"
if [ "\${1:-}" = "build" ]; then
    shift
    exec go build "-ldflags=-s -w -extar=$wrap_dir/zig-ar.sh" "\$@"
fi
exec go "\$@"
EOF

chmod +x \
    "$wrap_dir/zig-cc.sh" \
    "$wrap_dir/zig-cxx.sh" \
    "$wrap_dir/zig-ar.sh" \
    "$wrap_dir/zig-ranlib.sh" \
    "$wrap_dir/go-wrapper.sh"

cat > "$wrap_dir/musl_init_args.c" <<'EOF'
#include <stddef.h>
#include <stdint.h>

extern void _init(void);
extern char **__environ;
extern char *__progname_full;
extern __attribute__((weak, visibility("hidden"))) void (*const __init_array_start)(void);
extern __attribute__((weak, visibility("hidden"))) void (*const __init_array_end)(void);

void __libc_start_init(void) {
    int argc = 0;
    char **argv = NULL;
    char **envp = __environ;
    if (envp != NULL && envp[-1] == NULL) {
        char **end = envp - 1;
        for (char **cur = end - 1; (end - cur) <= 131072; --cur) {
            size_t candidate = (size_t)(end - cur);
            if (*(size_t *)(cur - 1) == candidate &&
                (cur[0] == __progname_full || candidate == 1)) {
                argc = (int)candidate;
                argv = cur;
                break;
            }
        }
    }
    _init();
    uintptr_t a = (uintptr_t)&__init_array_start;
    for (; a < (uintptr_t)&__init_array_end; a += sizeof(void (*)(void))) {
        (*(void (**)(int, char **, char **))a)(argc, argv, envp);
    }
}
EOF
"$wrap_dir/zig-cc.sh" -O2 -c "$wrap_dir/musl_init_args.c" -o "$wrap_dir/musl_init_args.o"

cat > "$wrap_dir/toolchain.cmake" <<EOF
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR $arch)
set(CMAKE_C_COMPILER "$wrap_dir/zig-cc.sh")
set(CMAKE_CXX_COMPILER "$wrap_dir/zig-cxx.sh")
set(CMAKE_ASM_COMPILER "$wrap_dir/zig-cc.sh")
set(CMAKE_AR "$wrap_dir/zig-ar.sh" CACHE FILEPATH "Archiver" FORCE)
set(CMAKE_RANLIB "$wrap_dir/zig-ranlib.sh" CACHE FILEPATH "Ranlib" FORCE)
EOF

bindgen_args="-isystem $zig_lib/include -isystem $zig_lib/libc/include/$zig_target -isystem $zig_lib/libc/include/generic-musl -isystem $zig_lib/libc/include/$zig_linux_inc -isystem $zig_lib/libc/include/any-linux-any"

target_env=$(printf '%s' "$target" | tr '-' '_')
target_upper=$(printf '%s' "$target_env" | tr '[:lower:]' '[:upper:]')

export GO="$wrap_dir/go-wrapper.sh"
export ROK_ALLOW_CROSS=1
export ROR_ALLOW_CROSS=1
export ROK_DISABLE_OPTIONAL_LIBS=1
export ROR_DISABLE_OPTIONAL_LIBS=1
export "CC_${target_env}=$wrap_dir/zig-cc.sh"
export "CXX_${target_env}=$wrap_dir/zig-cxx.sh"
export "AR_${target_env}=$wrap_dir/zig-ar.sh"
export "RANLIB_${target_env}=$wrap_dir/zig-ranlib.sh"
export "CMAKE_TOOLCHAIN_FILE_${target_env}=$wrap_dir/toolchain.cmake"
export "BINDGEN_EXTRA_CLANG_ARGS_${target_env}=$bindgen_args"
export "CARGO_TARGET_${target_upper}_LINKER=$wrap_dir/zig-cc.sh"
export "CARGO_TARGET_${target_upper}_RUSTFLAGS=-C link-self-contained=no -C link-arg=$wrap_dir/musl_init_args.o"

cd "$root"
exec cargo build --locked --profile dist \
    -p diorama --features diorama/oci-cri \
    -p dioramactl \
    -p diorama-addon \
    --target "$target"
