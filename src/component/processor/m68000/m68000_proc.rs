use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::TokenStream;
use quote::quote;

fn ax() -> &'static str { "self.a[(ir >> 9) as usize & 0x07]" }
fn ay() -> &'static str { "self.a[ir as usize & 0x07]" }
fn dx() -> &'static str { "self.d[(ir >> 9) as usize & 0x07]" }
fn dy() -> &'static str { "self.d[ir as usize & 0x07]" }

fn read_reg(mask: u32, source: &str, target: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        &format!("{} = {} & {};", target, source, mask)
    ).unwrap()
}

fn write_reg(mask: u32, source: &str, target: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        &format!("{} = {} & !{};", target, source, mask)
    ).unwrap()
}

fn imm(size: u32, target: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        &format!("{} = read_imm_{}!(self.pc);", target, size)
    ).unwrap()
}

fn read(size: u32, source: &str, target: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        &format!("{} = self.callback.read_memory_{}({});", target, size, source)
    ).unwrap()
}

fn write(size: u32, target: &str, source: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        &format!("self.callback.write_memory_{}({}, {});", size, target, source)
    ).unwrap()
}

fn ori_impl(size: u32, mask: u32) -> proc_macro2::TokenStream {
    quote! {
        dmdr |= smdr;
        self.nflag = dmdr >> (#size - 8);
        self.zflag = dmdr & #mask;
        self.cflag = SR_C_CLR;
        self.vflag = SR_V_CLR;
    }
}

// opcode, size, 
#[proc_macro]
pub fn ori(input: TokenStream) -> TokenStream {
    //           spec spec                   allowed ea mode      cpu cycles
    // name size proc ea   bit pattern       A+-DXWLdxI 0 1 2 4 000 010 020 040 instruction
    // ==== ===  ==== ==== ================= ========== = = = = === === === === ===================
    // ori, 16,  toc, .,   0000000000111100, .......... U U U U 20  16  12  12  ori.w #<data>, CCR
    // ori, 16,  tos, .,   0000000001111100, .......... S S S S 20  16  12  12  ori.w #<data>, SR
    // ori,  8,  .,   d,   0000000000000..., .......... U U U U  8   8   2   2  ori.b #<data>, Dn
    // ori,  8,  .,   .,   0000000000......, A+-DXWL... U U U U 12  12   4   4  ori.b #<data>, (An)
    // ori, 16,  .,   d,   0000000001000..., .......... U U U U  8   8   2   2  ori.w #<data>, Dn
    // ori, 16,  .,   .,   0000000001......, A+-DXWL... U U U U 12  12   4   4  ori.w #<data>, (An)
    // ori, 32,  .,   d,   0000000010000..., .......... U U U U 16  14   2   2  ori.l #<data>, Dn
    // ori, 32,  .,   .,   0000000010......, A+-DXWL... U U U U 20  20   4   4  ori.l #<data>, (An)
    // split input
    let input = input.to_string();
    let input = input.split(",").map(|x| x.trim()).collect::<Vec<&str>>();

    let mut output = proc_macro2::TokenStream::new();

    let size = u32::from_str(input[1]).unwrap();
    let mask = (1 << size) - 1;

    // fetch smar
    match input[2] {
        "ax" => output.extend(read_reg(mask, ax(), "smar")),
        "ay" => output.extend(read_reg(mask, ay(), "smar")),
        _ => {}
    }

    // fetch smdr
    match input[2] {
        "imm" => output.extend(imm(size, "smdr")),
        "ax" | "ay" => output.extend(read(size, "smar", "smdr")),
        _ => {}
    }

    // fetch dmar
    match input[3] {
        "ax" => output.extend(read_reg(mask, ax(), "dmar")),
        "ay" => output.extend(read_reg(mask, ay(), "dmar")),
        _ => {}
    }

    // fetch dmdr
    match input[3] {
        "ax" | "ay" => output.extend(read(size, "dmar", "dmdr")),
        "dx" => output.extend(read_reg(mask, dx(), "dmdr")),
        "dy" => output.extend(read_reg(mask, dy(), "dmdr")),
        _ => {}
    }

    match input[0] {
        "ori" => output.extend(ori_impl(size, mask)),
        _ => {}
    }

    match input[3] {
        "ax" | "ay" => output.extend(write(size, "dmar", "dmdr")),
        "dx" => output.extend(write_reg(mask, "dmdr", dx())),
        "dy" => output.extend(write_reg(mask, "dmdr", dy())),
        _ => {}
    }

    // {m68k_op_ori_8_d             , 0xfff8, 0x0000, {  8,   8,   2,   2}},
    // {m68k_op_ori_8_ai            , 0xfff8, 0x0010, { 16,  16,   8,   8}},
    // {m68k_op_ori_8_pi            , 0xfff8, 0x0018, { 16,  16,   8,   8}},
    // {m68k_op_ori_8_pd            , 0xfff8, 0x0020, { 18,  18,   9,   9}},
    // {m68k_op_ori_8_di            , 0xfff8, 0x0028, { 20,  20,   9,   9}},
    // {m68k_op_ori_8_ix            , 0xfff8, 0x0030, { 22,  22,  11,  11}},
    // {m68k_op_ori_16_d            , 0xfff8, 0x0040, {  8,   8,   2,   2}},
    // {m68k_op_ori_16_ai           , 0xfff8, 0x0050, { 16,  16,   8,   8}},
    // {m68k_op_ori_16_pi           , 0xfff8, 0x0058, { 16,  16,   8,   8}},
    // {m68k_op_ori_16_pd           , 0xfff8, 0x0060, { 18,  18,   9,   9}},
    // {m68k_op_ori_16_di           , 0xfff8, 0x0068, { 20,  20,   9,   9}},
    // {m68k_op_ori_16_ix           , 0xfff8, 0x0070, { 22,  22,  11,  11}},
    // {m68k_op_ori_32_d            , 0xfff8, 0x0080, { 16,  14,   2,   2}},
    // {m68k_op_ori_32_ai           , 0xfff8, 0x0090, { 28,  28,   8,   8}},
    // {m68k_op_ori_32_pi           , 0xfff8, 0x0098, { 28,  28,   8,   8}},
    // {m68k_op_ori_32_pd           , 0xfff8, 0x00a0, { 30,  30,   9,   9}},
    // {m68k_op_ori_32_di           , 0xfff8, 0x00a8, { 32,  32,   9,   9}},
    // {m68k_op_ori_32_ix           , 0xfff8, 0x00b0, { 34,  34,  11,  11}},
    //
    // {m68k_op_ori_8_pi7           , 0xffff, 0x001f, { 16,  16,   8,   8}},
    // {m68k_op_ori_8_pd7           , 0xffff, 0x0027, { 18,  18,   9,   9}},
    // {m68k_op_ori_8_aw            , 0xffff, 0x0038, { 20,  20,   8,   8}},
    // {m68k_op_ori_8_al            , 0xffff, 0x0039, { 24,  24,   8,   8}},
    // {m68k_op_ori_16_toc          , 0xffff, 0x003c, { 20,  16,  12,  12}},
    // {m68k_op_ori_16_aw           , 0xffff, 0x0078, { 20,  20,   8,   8}},
    // {m68k_op_ori_16_al           , 0xffff, 0x0079, { 24,  24,   8,   8}},
    // {m68k_op_ori_16_tos          , 0xffff, 0x007c, { 20,  16,  12,  12}},
    // {m68k_op_ori_32_aw           , 0xffff, 0x00b8, { 32,  32,   8,   8}},
    // {m68k_op_ori_32_al           , 0xffff, 0x00b9, { 36,  36,   8,   8}},
    
    TokenStream::from(
        quote! {
            {
                #output
            }
        }
    )
}

struct Instruction {
    mask: u32,
    pattern: u32,

    name: &'static str,
    size: u32,
    saddr: &'static str,
    daddr: &'static str,
}

#[proc_macro]
pub fn instructionSet(input: TokenStream) -> TokenStream {
    // op & 0xf100 == 0x7000 => op_moveq_32
    // [0x7000, 0x7eff]
    for _ in 0x7000..0x7eff {
    }

    // (op & 0xf000 == 0xa000)=> op_1010 (Unassigned, Reserved)
    // [0xa000, 0xafff] 
    for _ in 0xa000..0xafff {
    }

    // (op & 0xf000 == 0xf000)=> op_1111 Coprocessor Interface/MC68040 and CPU32 Extensions
    // [0xf000, 0xffff]
    for _ in 0xf000..0xffff {
    }

    // op & 0xf180 == 0xf080 => op_cpbcc_32 m68020+

    TokenStream::new()
}
