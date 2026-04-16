; AI-OS Bootloader
; Multiboot2 compatible bootloader for GRUB
; Switches to protected mode and loads the Rust kernel

section .multiboot_header
align 8
multiboot_header_start:
    ; Multiboot2 magic number
    dd 0xe85250d6                ; magic
    dd 0                         ; architecture (i386 protected mode)
    dd multiboot_header_end - multiboot_header_start  ; header length
    
    ; Checksum
    dd -(0xe85250d6 + 0 + (multiboot_header_end - multiboot_header_start))
    
    ; Optional multiboot tags
    align 8
    ; Framebuffer tag
    dw 5                         ; type = framebuffer
    dw 0                         ; flags
    dd 20                        ; size
    dd 1024                      ; width
    dd 768                       ; height
    dd 32                        ; depth
    
    ; End tag
    align 8
    dw 0                         ; type = end
    dw 0                         ; flags
    dd 8                         ; size
multiboot_header_end:

section .bss
align 16
stack_bottom:
    resb 16384                   ; 16 KB stack
stack_top:

section .text
bits 32
global _start

_start:
    ; Set up stack
    mov esp, stack_top
    
    ; Save Multiboot info pointer
    push ebx                     ; Multiboot info structure pointer
    push eax                     ; Multiboot magic number
    
    ; Check if we're loaded by a Multiboot-compliant bootloader
    cmp eax, 0x36d76289
    jne .no_multiboot
    
    ; Initialize early boot services
    call check_cpuid
    call check_long_mode
    call setup_page_tables
    call enable_paging
    
    ; Load GDT for 64-bit mode
    lgdt [gdt64.pointer]
    
    ; Jump to 64-bit code
    jmp gdt64.code:long_mode_start
    
.no_multiboot:
    ; Print error message
    mov dword [0xb8000], 0x4f524f45  ; 'ER' in red
    mov dword [0xb8004], 0x4f3a4f52  ; 'R:' in red
    mov dword [0xb8008], 0x4f204f20  ; '  ' in red
    mov dword [0xb800c], 0x4f4d4f4e  ; 'NM' in red
    mov dword [0xb8010], 0x4f204f42  ; 'B ' in red
    hlt

; Check if CPUID is supported
check_cpuid:
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21            ; Flip ID bit
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov dword [0xb8000], 0x4f524f45  ; Error: No CPUID
    hlt

; Check if long mode (64-bit) is available
check_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode
    
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29           ; Test LM bit
    jz .no_long_mode
    ret
.no_long_mode:
    mov dword [0xb8000], 0x4f324f36  ; Error: No 64-bit
    hlt

; Set up page tables for 64-bit mode
setup_page_tables:
    ; Map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11                ; Present + writable
    mov [p4_table], eax
    
    ; Map first P3 entry to P2 table
    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax
    
    ; Map each P2 entry to a 2MB page
    mov ecx, 0
.map_p2_table:
    mov eax, 0x200000           ; 2MB
    mul ecx
    or eax, 0b10000011          ; Present + writable + huge
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_table
    
    ret

; Enable paging
enable_paging:
    ; Load P4 table address
    mov eax, p4_table
    mov cr3, eax
    
    ; Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    
    ; Enable long mode
    mov ecx, 0xC0000080         ; EFER MSR
    rdmsr
    or eax, 1 << 8              ; Set LM bit
    wrmsr
    
    ; Enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    
    ret

; 64-bit code
bits 64
section .text
long_mode_start:
    ; Load null into all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Print "AI-OS" to screen
    mov rax, 0x2f4f2f49         ; 'I' 'O' in green
    mov [0xb8000], rax
    mov rax, 0x2f532f2d         ; '-' 'S' in green
    mov [0xb8004], rax
    
    ; Call Rust kernel
    extern rust_main
    call rust_main
    
    ; If kernel returns, halt
    .halt:
        hlt
        jmp .halt

; Global Descriptor Table for 64-bit mode
section .rodata
gdt64:
    dq 0                        ; Null descriptor
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; Code segment
.pointer:
    dw $ - gdt64 - 1           ; Length
    dq gdt64                    ; Address

; Page tables
section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
