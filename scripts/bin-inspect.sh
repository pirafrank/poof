#!/usr/bin/env bash
# bin-inspect.sh — Inspect binary file headers using only POSIX od(1).
# Works on Linux, macOS, and BSD without external tools.
#
# Usage: bin-inspect.sh <binary_file>

set -euo pipefail

usage() {
    echo "Usage: $(basename "$0") <binary_file>"
    echo ""
    echo "Outputs:"
    echo "  Magic number (all formats)"
    echo "  EI_CLASS, e_machine, and OS (ELF binaries)"
    echo "  cputype (Mach-O thin binaries)"
    echo "  cputypes (Mach-O FAT/Universal binaries)"
    exit 1
}

[[ $# -lt 1 ]] && usage

FILE="$1"
[[ ! -e "$FILE" ]] && { echo "Error: no such file: $FILE" >&2; exit 1; }
[[ ! -f "$FILE" ]] && { echo "Error: not a regular file: $FILE" >&2; exit 1; }
[[ ! -r "$FILE" ]] && { echo "Error: cannot read file: $FILE" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Low-level I/O helpers — all reads go through od(1) (POSIX, everywhere).
# ---------------------------------------------------------------------------

# _read_bytes OFFSET COUNT
# Prints space-separated decimal byte values; multiple od output lines are
# joined so callers always see a flat list.
_read_bytes() {
    od -A n -t u1 -j "$1" -N "$2" "$FILE" 2>/dev/null | tr '\n' ' '
}

# hex_str OFFSET COUNT
# Prints raw hex bytes concatenated with no separators, e.g. "7f454c46".
hex_str() {
    od -A n -t x1 -j "$1" -N "$2" "$FILE" 2>/dev/null | tr -d ' \n'
}

# u8 OFFSET → decimal
u8() {
    _read_bytes "$1" 1 | tr -d ' '
}

# u16_le OFFSET → decimal (little-endian)
u16_le() {
    local b
    b=($(_read_bytes "$1" 2))
    printf '%d' $(( b[0] | (b[1] << 8) ))
}

# u16_be OFFSET → decimal (big-endian)
u16_be() {
    local b
    b=($(_read_bytes "$1" 2))
    printf '%d' $(( (b[0] << 8) | b[1] ))
}

# u32_le OFFSET → decimal (little-endian)
u32_le() {
    local b
    b=($(_read_bytes "$1" 4))
    printf '%d' $(( b[0] | (b[1] << 8) | (b[2] << 16) | (b[3] << 24) ))
}

# u32_be OFFSET → decimal (big-endian)
u32_be() {
    local b
    b=($(_read_bytes "$1" 4))
    printf '%d' $(( (b[0] << 24) | (b[1] << 16) | (b[2] << 8) | b[3] ))
}

# ---------------------------------------------------------------------------
# ELF field decoders
# ---------------------------------------------------------------------------

ei_class_name() {
    case "$1" in
        1) echo "ELFCLASS32 (32-bit)" ;;
        2) echo "ELFCLASS64 (64-bit)" ;;
        *) echo "Unknown ($1)" ;;
    esac
}

e_machine_name() {
    case "$1" in
        0)   echo "EM_NONE" ;;
        2)   echo "EM_SPARC" ;;
        3)   echo "EM_386 (x86 32-bit)" ;;
        8)   echo "EM_MIPS" ;;
        20)  echo "EM_PPC (PowerPC 32-bit)" ;;
        21)  echo "EM_PPC64 (PowerPC 64-bit)" ;;
        40)  echo "EM_ARM (ARM 32-bit)" ;;
        62)  echo "EM_X86_64" ;;
        183) echo "EM_AARCH64 (ARM 64-bit)" ;;
        243) echo "EM_RISCV" ;;
        *)   echo "Unknown" ;;
    esac
}

# Round N up to the nearest multiple of 4 (for note field padding).
_align4() { echo $(( ($1 + 3) & ~3 )); }

# Decode the OS name from a GNU ABI tag desc[0] value.
_gnu_abi_os_name() {
    case "$1" in
        0) echo "Linux" ;;
        1) echo "Hurd" ;;
        2) echo "Solaris" ;;
        3) echo "FreeBSD" ;;
        4) echo "NetBSD" ;;
        5) echo "Syllable" ;;
        6) echo "NaCl" ;;
        *) echo "Unknown ($1)" ;;
    esac
}

# Scan ELF PT_NOTE segments and return the OS name found in notes.
# Arguments: ei_class (1=32-bit, 2=64-bit), endian ("little" or "big")
# Prints nothing if no known note is found.
_scan_elf_notes() {
    local ei_class="$1" endian="$2"

    # Read program header table location from the ELF header.
    # For 64-bit, e_phoff is an 8-byte field; we read only the low 32 bits
    # (sufficient for any binary under 4 GiB, which covers all real cases).
    local phoff phentsize phnum
    if [[ "$ei_class" == "1" ]]; then
        # ELF32: e_phoff at 28, e_phentsize at 42, e_phnum at 44
        if [[ "$endian" == "little" ]]; then
            phoff=$(u32_le 28); phentsize=$(u16_le 42); phnum=$(u16_le 44)
        else
            phoff=$(u32_be 28); phentsize=$(u16_be 42); phnum=$(u16_be 44)
        fi
    else
        # ELF64: e_phoff at 32 (8 bytes), e_phentsize at 54, e_phnum at 56
        # Low 32 bits: LE → offset 32, BE → offset 36 (MSB first)
        if [[ "$endian" == "little" ]]; then
            phoff=$(u32_le 32); phentsize=$(u16_le 54); phnum=$(u16_le 56)
        else
            phoff=$(u32_be 36); phentsize=$(u16_be 54); phnum=$(u16_be 56)
        fi
    fi

    local i phdr_off p_type p_offset p_filesz
    for (( i = 0; i < phnum; i++ )); do
        phdr_off=$(( phoff + i * phentsize ))

        if [[ "$endian" == "little" ]]; then
            p_type=$(u32_le "$phdr_off")
        else
            p_type=$(u32_be "$phdr_off")
        fi

        [[ "$p_type" != "4" ]] && continue  # PT_NOTE = 4

        # Read p_offset and p_filesz from this PT_NOTE entry.
        # ELF32 Phdr: p_offset at +4, p_filesz at +16
        # ELF64 Phdr: p_offset at +8 (8 bytes), p_filesz at +32 (8 bytes)
        #             Low 32 bits: LE → same offset, BE → offset+4
        if [[ "$ei_class" == "1" ]]; then
            if [[ "$endian" == "little" ]]; then
                p_offset=$(u32_le $(( phdr_off +  4 )))
                p_filesz=$(u32_le $(( phdr_off + 16 )))
            else
                p_offset=$(u32_be $(( phdr_off +  4 )))
                p_filesz=$(u32_be $(( phdr_off + 16 )))
            fi
        else
            if [[ "$endian" == "little" ]]; then
                p_offset=$(u32_le $(( phdr_off +  8 )))
                p_filesz=$(u32_le $(( phdr_off + 32 )))
            else
                p_offset=$(u32_be $(( phdr_off + 12 )))
                p_filesz=$(u32_be $(( phdr_off + 36 )))
            fi
        fi

        # Walk notes within this segment.
        # Each note: namesz(4) descsz(4) type(4) name(namesz) desc(descsz)
        # name and desc are each padded to a 4-byte boundary.
        local note_off note_end namesz descsz note_type pad_namesz pad_descsz name_hex
        note_off="$p_offset"
        note_end=$(( p_offset + p_filesz ))

        while (( note_off + 12 <= note_end )); do
            if [[ "$endian" == "little" ]]; then
                namesz=$(u32_le "$note_off")
                descsz=$(u32_le $(( note_off + 4 )))
                note_type=$(u32_le $(( note_off + 8 )))
            else
                namesz=$(u32_be "$note_off")
                descsz=$(u32_be $(( note_off + 4 )))
                note_type=$(u32_be $(( note_off + 8 )))
            fi

            # Sanity guard against corrupt/huge namesz values.
            if (( namesz == 0 || namesz > 256 )); then break; fi

            pad_namesz=$(_align4 "$namesz")
            pad_descsz=$(_align4 "$descsz")
            name_hex=$(hex_str $(( note_off + 12 )) "$namesz")

            case "$name_hex" in
                474e5500*)
                    # "GNU\0" — type 1 is NT_GNU_ABI_TAG; desc[0] holds the OS.
                    if [[ "$note_type" == "1" ]]; then
                        local desc_off os_val
                        desc_off=$(( note_off + 12 + pad_namesz ))
                        if [[ "$endian" == "little" ]]; then
                            os_val=$(u32_le "$desc_off")
                        else
                            os_val=$(u32_be "$desc_off")
                        fi
                        _gnu_abi_os_name "$os_val"
                        return
                    fi
                    ;;
                4672656542534400*)  # "FreeBSD\0"
                    echo "FreeBSD"; return ;;
                4f70656e42534400*)  # "OpenBSD\0"
                    echo "OpenBSD"; return ;;
                4e657442534400*)    # "NetBSD\0"
                    echo "NetBSD"; return ;;
                447261676f6e466c79*)  # "DragonFly..."
                    echo "DragonFly BSD"; return ;;
            esac

            note_off=$(( note_off + 12 + pad_namesz + pad_descsz ))
        done
    done
}

# Detect the target OS of an ELF binary.
# Checks EI_OSABI first; falls back to PT_NOTE scanning for ELFOSABI_NONE.
# Arguments: ei_class, endian
_detect_elf_os() {
    local ei_class="$1" endian="$2"
    local ei_osabi
    ei_osabi=$(u8 7)

    case "$ei_osabi" in
        0)  # ELFOSABI_NONE — most Linux and some BSD binaries; scan notes.
            local from_notes
            from_notes=$(_scan_elf_notes "$ei_class" "$endian")
            if [[ -n "$from_notes" ]]; then
                echo "$from_notes"
            else
                echo "System V / unknown (EI_OSABI=0, no identifying note found)"
            fi
            ;;
        1)  echo "HP-UX" ;;
        2)  echo "NetBSD" ;;
        3)  echo "Linux (GNU)" ;;
        4)  echo "Hurd" ;;
        6)  echo "Solaris" ;;
        7)  echo "AIX" ;;
        8)  echo "IRIX" ;;
        9)  echo "FreeBSD" ;;
        10) echo "Tru64" ;;
        12) echo "OpenBSD" ;;
        13) echo "OpenVMS" ;;
        64) echo "ARM EABI" ;;
        97) echo "ARM" ;;
        255) echo "Standalone / embedded" ;;
        *)  echo "Unknown (EI_OSABI=$ei_osabi)" ;;
    esac
}

handle_elf() {
    echo "Format:    ELF"

    local ei_class ei_data endian e_machine
    ei_class=$(u8 4)
    ei_data=$(u8 5)

    case "$ei_data" in
        1) endian="little" ;;
        2) endian="big" ;;
        *) endian="unknown" ;;
    esac

    printf "EI_CLASS:  %s\n" "$(ei_class_name "$ei_class")"
    printf "EI_DATA:   %s-endian\n" "$endian"

    if [[ "$endian" == "little" ]]; then
        e_machine=$(u16_le 18)
    else
        e_machine=$(u16_be 18)
    fi
    printf "e_machine: %s (0x%04x)\n" "$(e_machine_name "$e_machine")" "$e_machine"
    printf "OS:        %s\n" "$(_detect_elf_os "$ei_class" "$endian")"
}

# ---------------------------------------------------------------------------
# Mach-O field decoders
# ---------------------------------------------------------------------------

# CPU_TYPE_* constants from <mach/machine.h>
# CPU_ARCH_ABI64 = 0x01000000; values with that bit set are 64-bit variants.
macho_cpu_name() {
    case "$1" in
        7)        echo "i386" ;;
        16777223) echo "x86_64" ;;    # 0x01000007
        12)       echo "ARM" ;;
        16777228) echo "ARM64" ;;     # 0x0100000c
        16777229) echo "ARM64E" ;;    # 0x0100000d
        18)       echo "PowerPC" ;;
        16777234) echo "PowerPC64" ;; # 0x01000012
        *)        printf "Unknown (0x%x)" "$1" ;;
    esac
}

handle_macho_thin() {
    local endian="$1" bits="$2"
    printf "Format:    Mach-O thin (%s-bit, %s-endian)\n" "$bits" "$endian"

    local cputype
    if [[ "$endian" == "little" ]]; then
        cputype=$(u32_le 4)
    else
        cputype=$(u32_be 4)
    fi
    printf "cputype:   %s (0x%x)\n" "$(macho_cpu_name "$cputype")" "$cputype"
}

handle_macho_fat() {
    echo "Format:    Mach-O FAT (Universal Binary)"

    # fat_header is always big-endian on disk.
    local nfat_arch
    nfat_arch=$(u32_be 4)
    printf "nfat_arch: %d\n" "$nfat_arch"

    # Each fat_arch struct is 20 bytes; the table starts at offset 8.
    local i arch_offset cputype
    for (( i = 0; i < nfat_arch; i++ )); do
        arch_offset=$(( 8 + i * 20 ))
        cputype=$(u32_be "$arch_offset")
        printf "cputype[%d]: %s (0x%x)\n" "$i" "$(macho_cpu_name "$cputype")" "$cputype"
    done
}

# ---------------------------------------------------------------------------
# Main dispatcher
# ---------------------------------------------------------------------------

main() {
    local magic
    magic=$(hex_str 0 4)
    printf "Magic:     0x%s\n" "$magic"

    case "$magic" in
        7f454c46)
            handle_elf
            ;;
        feedface)
            handle_macho_thin "big" "32"
            ;;
        cefaedfe)
            handle_macho_thin "little" "32"
            ;;
        feedfacf)
            handle_macho_thin "big" "64"
            ;;
        cffaedfe)
            handle_macho_thin "little" "64"
            ;;
        cafebabe)
            handle_macho_fat
            ;;
        4d5a*)
            echo "Format:    PE/COFF (Windows Portable Executable)"
            ;;
        *)
            echo "Format:    Unknown"
            ;;
    esac
}

main
