; AI-OS UEFI Bootloader
; UEFI-compatible bootloader for modern systems

section .text
bits 64

global _start

; UEFI Entry Point
_start:
    ; RCX = EFI_HANDLE ImageHandle
    ; RDX = EFI_SYSTEM_TABLE *SystemTable
    
    ; Save UEFI handles
    mov [image_handle], rcx
    mov [system_table], rdx
    
    ; Set up stack
    mov rsp, stack_top
    
    ; Print boot message via UEFI
    call print_boot_message
    
    ; Get memory map from UEFI
    call get_memory_map
    
    ; Exit boot services
    call exit_boot_services
    
    ; Initialize our own page tables
    call setup_kernel_page_tables
    
    ; Jump to Rust kernel
    extern rust_main
    call rust_main
    
    ; Should never reach here
    .halt:
        hlt
        jmp .halt

print_boot_message:
    push rbx
    push rcx
    push rdx
    
    ; Get console output protocol
    mov rax, [system_table]
    mov rax, [rax + 64]         ; ConOut
    
    ; OutputString(ConOut, String)
    lea rdx, [boot_message]
    mov rcx, rax
    mov rax, [rcx]
    call [rax + 8]              ; OutputString function
    
    pop rdx
    pop rcx
    pop rbx
    ret

get_memory_map:
    push rbx
    push rcx
    push rdx
    
    ; Call GetMemoryMap
    mov rax, [system_table]
    mov rax, [rax + 96]         ; BootServices
    
    ; Parameters for GetMemoryMap
    lea rcx, [memory_map_size]
    lea rdx, [memory_map]
    lea r8, [memory_map_key]
    lea r9, [descriptor_size]
    push descriptor_version
    
    mov rbx, [rax + 56]         ; GetMemoryMap function
    call rbx
    
    add rsp, 8
    pop rdx
    pop rcx
    pop rbx
    ret

exit_boot_services:
    push rbx
    push rcx
    push rdx
    
    ; ExitBootServices(ImageHandle, MapKey)
    mov rcx, [image_handle]
    mov rdx, [memory_map_key]
    
    mov rax, [system_table]
    mov rax, [rax + 96]         ; BootServices
    mov rbx, [rax + 232]        ; ExitBootServices function
    call rbx
    
    pop rdx
    pop rcx
    pop rbx
    ret

setup_kernel_page_tables:
    ; Identity map first 4GB
    ; Set up PML4
    mov rax, pdp_table
    or rax, 0x3                 ; Present + Writable
    mov [pml4_table], rax
    
    ; Set up PDP
    mov rcx, 0
.loop_pdp:
    mov rax, rcx
    shl rax, 30                 ; 1GB pages
    or rax, 0x83                ; Present + Writable + Huge
    mov [pdp_table + rcx * 8], rax
    inc rcx
    cmp rcx, 4                  ; Map 4GB
    jl .loop_pdp
    
    ; Load new page tables
    mov rax, pml4_table
    mov cr3, rax
    
    ret

section .data
boot_message:
    dw 'A', 0, 'I', 0, '-', 0, 'O', 0, 'S', 0, ' ', 0
    dw 'B', 0, 'o', 0, 'o', 0, 't', 0, 'i', 0, 'n', 0, 'g', 0, '.', 0, '.', 0, '.', 0
    dw 0x0D, 0, 0x0A, 0        ; CR LF
    dw 0, 0                     ; Null terminator

section .bss
align 16
stack_bottom:
    resb 65536                  ; 64 KB stack
stack_top:

image_handle:
    resq 1
system_table:
    resq 1

memory_map_size:
    resq 1
memory_map:
    resb 4096
memory_map_key:
    resq 1
descriptor_size:
    resq 1
descriptor_version:
    resq 1

; Page tables
align 4096
pml4_table:
    resb 4096
pdp_table:
    resb 4096
