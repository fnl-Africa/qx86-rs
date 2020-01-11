use crate::vm::*;
use crate::pipeline::*;
use crate::structs::*;
use crate::flags::X86Flags;
use bitvec::prelude::{
    BitStore,
    BigEndian,
    LittleEndian,
};

/// The logic function for the `mov` opcode
pub fn mov(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    vm.set_arg(pipeline.args[0].location, vm.get_arg(pipeline.args[1].location)?)?;
    Ok(())
}
/// The logic function for the 'pusha' opcode
pub fn pusha(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override {
        let sp = vm.get_reg(Reg16::SP as u8, ValueSize::Word);
        vm.push_stack(vm.get_reg(Reg16::AX as u8, ValueSize::Word), pipeline)?;
        vm.push_stack(vm.get_reg(Reg16::CX as u8, ValueSize::Word), pipeline)?;
        vm.push_stack(vm.get_reg(Reg16::DX as u8, ValueSize::Word), pipeline)?;
        vm.push_stack(vm.get_reg(Reg16::BX as u8, ValueSize::Word), pipeline)?;
        vm.push_stack(sp, pipeline)?;
        vm.push_stack(vm.get_reg(Reg16::BP as u8, ValueSize::Word), pipeline)?;
        vm.push_stack(vm.get_reg(Reg16::SI as u8, ValueSize::Word), pipeline)?;
        return vm.push_stack(vm.get_reg(Reg16::DI as u8, ValueSize::Word), pipeline);
    } else {
        let esp = vm.get_reg(Reg32::ESP as u8, ValueSize::Dword);
        vm.push_stack(vm.get_reg(Reg32::EAX as u8, ValueSize::Dword), pipeline)?;
        vm.push_stack(vm.get_reg(Reg32::ECX as u8, ValueSize::Dword), pipeline)?;
        vm.push_stack(vm.get_reg(Reg32::EDX as u8, ValueSize::Dword), pipeline)?;
        vm.push_stack(vm.get_reg(Reg32::EBX as u8, ValueSize::Dword), pipeline)?;
        vm.push_stack(esp, pipeline)?;
        vm.push_stack(vm.get_reg(Reg32::EBP as u8, ValueSize::Dword), pipeline)?;
        vm.push_stack(vm.get_reg(Reg32::ESI as u8, ValueSize::Dword), pipeline)?;
        return vm.push_stack(vm.get_reg(Reg32::EDI as u8, ValueSize::Dword), pipeline);
    }
}
/// The logic function for the 'popa' opcode
pub fn popa(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let di = vm.pop16()?;
        let si = vm.pop16()?;
        let bp = vm.pop16()?;
        vm.pop16()?; //sp doesn't get set
        let bx = vm.pop16()?;
        let dx = vm.pop16()?;
        let cx = vm.pop16()?;
        let ax = vm.pop16()?;
        vm.set_reg(Reg16::DI as u8, di);
        vm.set_reg(Reg16::SI as u8, si);
        vm.set_reg(Reg16::BP as u8, bp);
        vm.set_reg(Reg16::BX as u8, bx);
        vm.set_reg(Reg16::DX as u8, dx);
        vm.set_reg(Reg16::CX as u8, cx);
        vm.set_reg(Reg16::AX as u8, ax);
    } else {
        let edi = vm.pop32()?;
        let esi = vm.pop32()?;
        let ebp = vm.pop32()?;
        vm.pop32()?; //esp doesn't get set
        let ebx = vm.pop32()?;
        let edx = vm.pop32()?;
        let ecx = vm.pop32()?;
        let eax = vm.pop32()?;
        vm.set_reg(Reg32::EDI as u8, edi);
        vm.set_reg(Reg32::ESI as u8, esi);
        vm.set_reg(Reg32::EBP as u8, ebp);
        vm.set_reg(Reg32::EBX as u8, ebx);
        vm.set_reg(Reg32::EDX as u8, edx);
        vm.set_reg(Reg32::ECX as u8, ecx);
        vm.set_reg(Reg32::EAX as u8, eax);
    }
    Ok(())
}
///  The logic function for the 'enter' opcode
pub fn enter(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let locals = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let mut nesting = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    //push ebp
    let mut ebp = vm.get_reg(Reg32::EBP as u8, ValueSize::Dword).u32_exact()?;
    vm.push_stack(SizedValue::Dword(ebp), pipeline)?;
    //temp . esp
    let temp = vm.get_reg(Reg32::ESP as u8, ValueSize::Dword);
    // todo: This portion appears to be erroring but I can't figure out why
    // should return and fix this in the future
    // while (nesting > 0)
    //  nesting . nesting - 1
    //  eBP . eBP - n
    // push [SS:eBP]
    while nesting > 0 {
        unimplemented!("Nesting is not working properly yet. TBC.");
        nesting = nesting - 1;
        ebp = ebp - 4;
        vm.push_stack(SizedValue::Dword(ebp), pipeline)?;
    }
    //ebp . temp
    vm.set_reg(Reg32::EBP as u8, temp);
    //esp . esp - locals
    let esp = temp.u32_exact()?;
    let (result, _) = esp.overflowing_sub(locals as u32);
    vm.set_reg(Reg32::ESP as u8, SizedValue::Dword(result));
    Ok(())
}
///  The logic function for the 'leave' opcode
pub fn leave(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //mov esp, ebp
    let ebp = vm.get_reg(Reg32::EBP as u8, ValueSize::Dword);
    vm.set_reg(Reg32::ESP as u8, ebp);
    //pop ebp
    let dword = vm.pop32()?;
    vm.set_reg(Reg32::EBP as u8, dword);
    Ok(())
}
/// The logic function for the `push` opcode
pub fn push(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let v = vm.get_arg(pipeline.args[0].location)?;
    vm.push_stack(v, pipeline)?;
    Ok(())
}
/// The logic function for the `pop` opcode
pub fn pop(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //Important edge case:
    /* https://c9x.me/x86/html/file_module_x86_id_248.html
    If the ESP register is used as a base register for addressing a destination operand in memory, 
    the POP instruction computes the effective address of the operand after it increments the ESP register.

    The POP ESP instruction increments the stack pointer (ESP) before data at the old top of stack is written into the destination
    */
    if pipeline.size_override{
        let word = vm.pop16()?;
        vm.set_arg(pipeline.args[0].location, word)?;
    }else{
        let dword = vm.pop32()?;
        vm.set_arg(pipeline.args[0].location, dword)?;
    };
    Ok(())
}
/// The logic for the `bswap` opcode
pub fn bswap(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
        let source = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Dword(source.swap_bytes()))?;
        Ok(())
}

pub fn pushf(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let flag_int = vm.flags.serialize_flag_storage();
    vm.push_stack(SizedValue::Dword(flag_int), pipeline)?;
    Ok(())
}

pub fn popf(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let flag_result = vm.pop32()?;
    let flag_int = flag_result.u32_exact()?;
    vm.flags.deserialize_flag_storage(flag_int);
    Ok(())
}

pub fn lahf(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let flag_int = vm.flags.serialize_flag_storage();
    vm.set_reg(Reg8::AH as u8, SizedValue::Byte(flag_int as u8));
    Ok(())
}

pub fn sahf(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let ah = vm.get_reg(Reg8::AH as u8, ValueSize::Byte).u8_exact()?;
    vm.flags.deserialize_flag_storage(ah as u32);
    Ok(())
}

pub fn cbw_cwde(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let lower_half = vm.reg8(Reg8::AL) as u8;
        if lower_half < 0x80 {
            vm.set_reg(Reg8::AH as u8, SizedValue::Byte(0x0));
        } else {
            vm.set_reg(Reg8::AH as u8, SizedValue::Byte(0xFF));
        }
    } else {
        let lower_half = vm.reg16(Reg16::AX) as u16;
        if lower_half > 0x8000 {
            vm.set_reg(Reg32::EAX as u8, SizedValue::Dword(0xFFFF0000 + lower_half as u32));
        } else {
            vm.set_reg(Reg32::EAX as u8, SizedValue::Dword(lower_half as u32));            
        }
    }
    Ok(())
}

pub fn cdq_cwd(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let lower_half = vm.reg16(Reg16::AX) as u16;
        if lower_half < 0x8000 {
            vm.set_reg(Reg16::DX as u8, SizedValue::Word(0x0));
        } else {
            vm.set_reg(Reg16::DX as u8, SizedValue::Word(0xFFFF));
        }
    } else {
        let lower_half = vm.reg32(Reg32::EAX) as u32;
        if lower_half < 0x80000000 {
            vm.set_reg(Reg32::EDX as u8, SizedValue::Dword(0x0));   
        } else {
            vm.set_reg(Reg32::EDX as u8, SizedValue::Dword(0xFFFFFFFF));
        }
    }
    Ok(())
}

pub fn bit_test(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let bitset = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 16) as u8).into()) as bool;
    } else {
        let bitset = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 32) as u8).into()) as bool;
    }
    Ok(())
}

pub fn bit_test_set(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 16) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 16) as u8).into(), true);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Word(bitset));          
    } else {
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 32) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 32) as u8).into(), true);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Dword(bitset));    
    }
}

pub fn bit_test_reset(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 16) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 16) as u8).into(), false);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Word(bitset));          
    } else {
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u32_exact()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 32) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 32) as u8).into(), false);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Dword(bitset));    
    }
}


pub fn bit_test_complement(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 16) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 16) as u8).into(), !vm.flags.carry);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Word(bitset));          
    } else {
        let mut bitset = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let index = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
        vm.flags.carry = bitset.get::<LittleEndian>(((index % 32) as u8).into()) as bool;
        bitset.set::<LittleEndian>(((index % 32) as u8).into(), !vm.flags.carry);
        return vm.set_arg(pipeline.args[0].location, SizedValue::Dword(bitset));    
    }
}

pub fn bit_scan_forward(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        let source = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let mut index: Option<u16> = None;
        for i in 0..16 {
            if source.get::<LittleEndian>(i.into()){
                index = Some(i.into()); 
                break;
            }
        }
        if index.is_none() {
            vm.flags.zero = true;            
        } else {
            vm.flags.zero = false;
            return vm.set_arg(pipeline.args[1].location, SizedValue::Word(index.unwrap()));
        }
    } else {
        let source = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let mut index: Option<u32> = None;
        for i in 0..32 {
            if source.get::<LittleEndian>(i.into()){
                index = Some(i.into()); 
                break;
            }
        }
        if index.is_none() {
            vm.flags.zero = true;            
        } else {
            vm.flags.zero = false;
            return vm.set_arg(pipeline.args[1].location, SizedValue::Dword(index.unwrap()));
        }
    }
    Ok(())
}

pub fn bit_scan_reverse(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        let source = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let mut index: Option<u16> = None;
        for i in 0..16 {
            if source.get::<BigEndian>(i.into()){
                index = Some(i.into()); 
                break;
            }
        }
        if index.is_none() {
            vm.flags.zero = true;            
        } else {
            vm.flags.zero = false;
            return vm.set_arg(pipeline.args[1].location, SizedValue::Word(15 - index.unwrap()));
        }
    } else {
        let source = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let mut index: Option<u32> = None;
        for i in 0..32 {
            if source.get::<BigEndian>(i.into()){
                index = Some(i.into()); 
                break;
            }
        }
        if index.is_none() {
            vm.flags.zero = true;            
        } else {
            vm.flags.zero = false;
            return vm.set_arg(pipeline.args[1].location, SizedValue::Dword(31 - index.unwrap()));
        }
    }
    Ok(())
}

pub fn xchg(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let source = vm.get_arg(pipeline.args[0].location)?;
    let destination = vm.get_arg(pipeline.args[1].location)?;
    vm.set_arg(pipeline.args[0].location, destination)?;
    vm.set_arg(pipeline.args[1].location, source)?;
    Ok(())
}

pub fn ret(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let stack_clear = vm.get_arg(pipeline.args[0].location)?.u16_zx()?;
    if pipeline.size_override{
        let word = vm.pop16()?;
        vm.eip = (word.u32_zx()? - (pipeline.eip_size as u32)) & 0xFFFF;
    }else{
        let dword = vm.pop32()?;
        vm.eip = dword.u32_zx()? - (pipeline.eip_size as u32);
    };
    if stack_clear != 0 {
        vm.regs[Reg32::ESP as usize] += stack_clear as u32;
    }
    Ok(())
}

pub fn call_rel(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let branch_to = vm.get_arg(pipeline.args[0].location)?.u32_zx()?;
    vm.push_stack(SizedValue::Dword(vm.eip + pipeline.eip_size as u32), pipeline)?;
    vm.set_arg(pipeline.args[1].location, SizedValue::Dword(branch_to))?;
    jmp_rel(vm, pipeline, _hv)?;
    Ok(())
}

pub fn call_abs(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let branch_to = vm.get_arg(pipeline.args[0].location)?.u32_zx()?;
    vm.push_stack(SizedValue::Dword(vm.eip + pipeline.eip_size as u32), pipeline)?;
    vm.set_arg(pipeline.args[1].location, SizedValue::Dword(branch_to))?;
    jmp_abs(vm, pipeline, _hv)?;
    Ok(())
}

/// The logic function for the `jmp` opcodes with a relative argument
pub fn jmp_rel(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //relative jumps are calculated from the EIP value AFTER the jump would've executed, ie, after EIP is advanced by the size of the instruction
    let future_eip = vm.eip + (pipeline.eip_size as u32);
    //rel must be sign extended, but is otherwise treated as a u32 for simplicity
    //an i32 and a u32 will behave the same way for wrapping_addition like this
    let rel = vm.get_arg(pipeline.args[0].location)?.u32_sx()?;
    //subtract out the eip_size that'll be advanced in the cycle() main loop
    vm.eip = future_eip.wrapping_add(rel) - (pipeline.eip_size as u32);
    if pipeline.size_override{
        vm.eip &= 0xFFFF;
    }
    Ok(())
}
/// The logic function for the `jmp` opcodes with an absolute argument
pub fn jmp_abs(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //must subtract the size of this opcode to correct for the automatic eip_size advance in the cycle() main loop
    vm.eip = vm.get_arg(pipeline.args[0].location)?.u32_zx()? - (pipeline.eip_size as u32);
    if pipeline.size_override{
        vm.eip &= 0xFFFF;
    }
    Ok(())
}

pub fn jmp_conditional_ecx_is_zero(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if vm.regs[Reg32::ECX as usize] == 0 {
        return jmp_rel(vm, pipeline, _hv);
    }
    Ok(())
}

fn cc_matches(opcode: u8, flags: &X86Flags) -> bool{
    let cc = 0x0F & opcode;
    match cc{
        0x0 => flags.overflow,
        0x1 => !flags.overflow,
        0x2 => flags.carry,
        0x3 => !flags.carry,
        0x4 => flags.zero,
        0x5 => !flags.zero,
        0x6 => flags.carry | flags.zero,
        0x7 => !flags.carry & !flags.zero,
        0x8 => flags.sign,
        0x9 => !flags.sign,
        0xA => flags.parity,
        0xB => !flags.parity,
        0xC => flags.sign != flags.overflow,
        0xD => flags.sign == flags.overflow,
        0xE => (flags.sign != flags.overflow) | flags.zero,
        0xF => (flags.sign == flags.overflow) & !flags.zero,
        _ => false
    }
}

pub fn jcc(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if cc_matches(pipeline.opcode, &vm.flags){
        return jmp_rel(vm, pipeline, _hv);
    }
    Ok(())
}

pub fn div_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg16(Reg16::AX) as u16;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u16_zx()?;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte((first_arg / second_arg) as u8));
    vm.set_reg(Reg8::AH as u8, SizedValue::Byte((first_arg%second_arg) as u8));
    Ok(())
}

pub fn div_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return div_16bit(vm, pipeline, _hv);
    } else {
        return div_32bit(vm, pipeline, _hv);
    }
}

pub fn div_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = ((vm.reg16(Reg16::DX) as u32) << 16) | (vm.reg16(Reg16::AX) as u32);
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()?;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFFFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg16::AX as u8, SizedValue::Word((first_arg / second_arg) as u16));
    vm.set_reg(Reg16::DX as u8, SizedValue::Word((first_arg%second_arg) as u16));
    Ok(())
}

pub fn div_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = ((vm.reg32(Reg32::EDX) as u64) << 32) | (vm.reg32(Reg32::EAX) as u64);
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()? as u64;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFFFFFFFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg32::EAX as u8, SizedValue::Dword((first_arg / second_arg) as u32));
    vm.set_reg(Reg32::EDX as u8, SizedValue::Dword((first_arg%second_arg) as u32));
    Ok(())
}

pub fn idiv_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg16(Reg16::AX) as i16;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u16_zx()? as i16;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte((first_arg / second_arg) as u8));
    vm.set_reg(Reg8::AH as u8, SizedValue::Byte((first_arg%second_arg) as u8));
    Ok(())
}

pub fn idiv_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return idiv_16bit(vm, pipeline, _hv);
    } else {
        return idiv_32bit(vm, pipeline, _hv);
    }
}

pub fn idiv_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = ((vm.reg16(Reg16::DX) as i16 as i32) << 16) | (vm.reg16(Reg16::AX) as u16 as i32);
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()? as i32;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFFFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg16::AX as u8, SizedValue::Word((first_arg / second_arg) as u16));
    vm.set_reg(Reg16::DX as u8, SizedValue::Word((first_arg%second_arg) as u16));
    Ok(())
}

pub fn idiv_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = ((vm.reg32(Reg32::EDX) as i32 as i64) << 32) | (vm.reg32(Reg32::EAX) as i32 as i64);
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()? as i32 as i64;
    if second_arg == 0 || ((first_arg / second_arg) > 0xFFFFFFFF) {
        return Err(VMError::DivideByZero) // divide by 0 not allowed and result being too big for destination not allowed
    }
    vm.set_reg(Reg32::EAX as u8, SizedValue::Dword((first_arg / second_arg) as u32));
    vm.set_reg(Reg32::EDX as u8, SizedValue::Dword((first_arg%second_arg) as u32));
    Ok(())
}

pub fn mul_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg8(Reg8::AL) as u16;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u16_zx()?;
    let result = first_arg.wrapping_mul(second_arg);
    if result & 0xFF00 > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_reg(Reg16::AX as u8, SizedValue::Word(result));
    Ok(())
}

pub fn mul_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return mul_16bit(vm, pipeline, _hv);
    } else {
        return mul_32bit(vm, pipeline, _hv);
    }
}

pub fn mul_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg16(Reg16::AX) as u32;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()?;
    let result = first_arg.wrapping_mul(second_arg);
    vm.set_reg(Reg16::AX as u8, SizedValue::Word((result&0x0000FFFF) as u16));
    vm.set_reg(Reg16::DX as u8, SizedValue::Word(((result&0xFFFF0000).wrapping_shr(16)) as u16));
    if vm.reg16(Reg16::DX) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    Ok(())
}

pub fn mul_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg32(Reg32::EAX) as u64;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_zx()? as u64;
    let result = first_arg.wrapping_mul(second_arg);
    vm.set_reg(Reg32::EAX as u8, SizedValue::Dword((result&0x00000000FFFFFFFF) as u32));
    vm.set_reg(Reg32::EDX as u8, SizedValue::Dword(((result&0xFFFFFFFF00000000).wrapping_shr(32)) as u32));
    if vm.reg32(Reg32::EDX) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    Ok(())
}

pub fn imul1_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg8(Reg8::AL) as i16;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u16_sx()? as i16;
    let result = (first_arg.wrapping_mul(second_arg)) as u16;
    if result & 0xFF00 > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_reg(Reg16::AX as u8, SizedValue::Word(result));
    Ok(())
}

pub fn imul1_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return imul1_16bit(vm, pipeline, _hv);
    } else {
        return imul1_32bit(vm, pipeline, _hv);
    }
}

pub fn imul1_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg16(Reg16::AX) as i16 as i32;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u16_sx()? as i16 as i32;
    let result = (first_arg.wrapping_mul(second_arg)) as u32;
    vm.set_reg(Reg16::AX as u8, SizedValue::Word((result&0x0000FFFF) as u16));
    vm.set_reg(Reg16::DX as u8, SizedValue::Word(((result&0xFFFF0000).wrapping_shr(16)) as u16));
    if vm.reg16(Reg16::DX) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    Ok(())
}

pub fn imul1_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.reg32(Reg32::EAX) as i32 as i64;
    let second_arg = vm.get_arg(pipeline.args[0].location)?.u32_sx()? as i32 as i64;
    let result = (first_arg.wrapping_mul(second_arg)) as u64;
    vm.set_reg(Reg32::EAX as u8, SizedValue::Dword((result&0x00000000FFFFFFFF) as u32));
    vm.set_reg(Reg32::EDX as u8, SizedValue::Dword(((result&0xFFFFFFFF00000000).wrapping_shr(32)) as u32));
    if vm.reg32(Reg32::EDX) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    Ok(())
}

pub fn imul2_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return imul2_16bit(vm, pipeline, _hv);
    } else {
        return imul2_32bit(vm, pipeline, _hv);
    }
}

pub fn imul2_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.get_arg(pipeline.args[0].location)?.u16_sx()? as i16 as i32;
    let second_arg = vm.get_arg(pipeline.args[1].location)?.u16_sx()? as i16 as i32;
    let result = (first_arg.wrapping_mul(second_arg)) as u32;
    if (result&0xFFFF0000).wrapping_shr(16) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_arg(pipeline.args[0].location, SizedValue::Word((result&0x0000FFFF) as u16))?;
    Ok(())
}

pub fn imul2_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.get_arg(pipeline.args[0].location)?.u32_sx()? as i32 as i64;
    let second_arg = vm.get_arg(pipeline.args[1].location)?.u32_sx()? as i32 as i64;
    let result = (first_arg.wrapping_mul(second_arg)) as u64;
    if (result&0xFFFFFFFF00000000).wrapping_shr(16) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword((result&0x00000000FFFFFFFF) as u32))?;
    Ok(())
}

pub fn imul3_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return imul3_16bit(vm, pipeline, _hv);
    } else {
        return imul3_32bit(vm, pipeline, _hv);
    }
}

pub fn imul3_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.get_arg(pipeline.args[1].location)?.u32_sx()? as i32 as i64;
    let second_arg = vm.get_arg(pipeline.args[2].location)?.u32_sx()? as i32 as i64;
    let result = (first_arg.wrapping_mul(second_arg)) as u32;
    if (result&0xFFFF0000).wrapping_shr(16) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_arg(pipeline.args[0].location, SizedValue::Word((result&0x0000FFFF) as u16))?;
    Ok(())
}

pub fn imul3_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let first_arg = vm.get_arg(pipeline.args[1].location)?.u32_sx()? as i32 as i64;
    let second_arg = vm.get_arg(pipeline.args[2].location)?.u32_sx()? as i32 as i64;
    let result = (first_arg.wrapping_mul(second_arg)) as u64;
    if (result&0xFFFFFFFF00000000).wrapping_shr(16) > 0 {
        vm.flags.carry = true;
        vm.flags.overflow = true;
    } else {
        vm.flags.carry = false;
        vm.flags.overflow = false;
    }
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword((result&0x00000000FFFFFFFF) as u32))?;
    Ok(())
}

pub fn shl_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result= (destination as u16) << count;
    if count == 1 {
        vm.flags.overflow = (destination & 0x80) != ((result as u8) & 0x80);
    }
    vm.flags.carry = result & 0x100 != 0;
    vm.flags.calculate_zero(result as u8 as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign8(result as u8);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn shl_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return shl_16bit(vm, pipeline, _hv);
    } else {
        return shl_32bit(vm, pipeline, _hv);
    }
}

pub fn shl_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result= (destination as u32) << count;
    if count == 1 {
        vm.flags.overflow = ((destination as u32) & 0x8000) != (result & 0x8000);
    }
    vm.flags.carry = (result & 0x1000) != 0;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign16(result as u16);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn shl_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result= (destination as u64) << count;
    if count == 1 {
        vm.flags.overflow = ((destination as u64) & 0x80000000) != (result & 0x80000000);
    }
    vm.flags.carry = (result & 0x100000000) != 0;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign32(result as u32);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn shr_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result= destination >> count;
    let computation_result = destination >> (count - 1);
    if count == 1 {
        vm.flags.overflow = (destination & 0x80) != (result & 0x80);
    }
    vm.flags.carry = computation_result & 1 != 0;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign8(result);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn shr_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return shr_16bit(vm, pipeline, _hv);
    } else {
        return shr_32bit(vm, pipeline, _hv);
    }
}

pub fn shr_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result= (destination as u32) >> count;
    let computation_result = (destination as u32) >> (count - 1);
    if count == 1 {
        vm.flags.overflow = ((destination as u32) & 0x8000) != (result & 0x8000);
    }
    vm.flags.carry = (computation_result & 1) != 0;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign16(result as u16);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn shr_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let destination = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let count = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result= (destination as u64) >> count;
    let computation_result = destination >> (count - 1);
    if count == 1 {
        vm.flags.overflow = ((destination as u64) & 0x80000000) != (result & 0x80000000);
    }
    vm.flags.carry = (computation_result & 1) != 0;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign32(result as u32);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn aaa(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let mut al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let ah = vm.get_reg(Reg8::AH as u8, ValueSize::Byte).u8_exact()?;
    if (al & 0x0F) > 9 || vm.flags.adjust {
        vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al.wrapping_add(6)));
        vm.set_reg(Reg8::AH as u8, SizedValue::Byte(ah.wrapping_add(1)));
        vm.flags.adjust = true;
        vm.flags.carry = true;
    } else {
        vm.flags.adjust = false;
        vm.flags.carry = false;
    }
    al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al & 0x0F));
    Ok(())
}

pub fn aas(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let mut al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let ah = vm.get_reg(Reg8::AH as u8, ValueSize::Byte).u8_exact()?;
    if ((al & 0x0F) > 9) || vm.flags.adjust {
        vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al.wrapping_sub(6)));
        vm.set_reg(Reg8::AH as u8, SizedValue::Byte(ah.wrapping_sub(1)));
        vm.flags.adjust = true;
        vm.flags.carry = true;
    } else {
        vm.flags.adjust = false;
        vm.flags.carry = false;
    }
    al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al & 0x0F));
    Ok(())
}

pub fn daa(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let old_al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let mut al = old_al;
    let old_carry = vm.flags.carry;
    if (al & 0x0F) > 9 || vm.flags.adjust{
        let (temp_al, carry) = al.overflowing_add(6);
        al = temp_al;
        vm.flags.carry = carry;
        vm.flags.adjust = true;
    } else {
        vm.flags.adjust = false;
    }
    if (old_al > 0x99) || old_carry {
        let (temp_al, _carry) = al.overflowing_add(0x60);
        al = temp_al;
        vm.flags.carry = true;
    } else {
        vm.flags.carry = false;
    }
    vm.flags.calculate_parity(al as u32);
    vm.flags.calculate_sign8(al);
    vm.flags.calculate_zero(al as u32);
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al));
    Ok(())
}

pub fn das(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let old_al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let mut al = old_al;
    let old_carry = vm.flags.carry;
    if (al & 0x0F) > 9 || vm.flags.adjust{
        let (temp_al, carry) = al.overflowing_sub(6);
        al = temp_al;
        vm.flags.carry = carry;
        vm.flags.adjust = true;
    } else {
        vm.flags.adjust = false;
    }
    if (old_al > 0x99) || old_carry {
        let (temp_al, _carry) = al.overflowing_sub(0x60);
        al = temp_al;
        vm.flags.carry = true;
    }
    vm.flags.calculate_parity(al as u32);
    vm.flags.calculate_sign8(al);
    vm.flags.calculate_zero(al as u32);
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(al));
    Ok(())
}

pub fn aad(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let ah = vm.get_reg(Reg8::AH as u8, ValueSize::Byte).u8_exact()?;
    let second_byte = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let result = ah.wrapping_mul(second_byte).wrapping_add(al);
    vm.flags.calculate_sign8(result);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_zero(result as u32);    
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(result));
    vm.set_reg(Reg8::AH as u8, SizedValue::Byte(0));
    Ok(())
}

pub fn aam(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let al = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let ah = vm.get_reg(Reg8::AH as u8, ValueSize::Byte).u8_exact()?;
    let second_byte = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    if second_byte == 0 {
        return Err(VMError::DivideByZero);
    }
    let result = al / second_byte;
    let second_result = al % second_byte;
    vm.flags.calculate_sign8(second_result);
    vm.flags.calculate_parity(second_result as u32);
    vm.flags.calculate_zero(second_result as u32);
    vm.set_reg(Reg8::AH as u8, SizedValue::Byte(result));
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(second_result));
    Ok(())
}

pub fn adc_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let carry_add = if vm.flags.carry{
        1
    }else{
        0
    };
    let prelim_sum = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(prelim_sum.wrapping_add(carry_add)))?;    
    return add_8bit(vm, pipeline, _hv);
}

pub fn adc_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let carry_add = if vm.flags.carry{
        1
    }else{
        0
    };
    if pipeline.size_override{
        let prelim_sum = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Word(prelim_sum.wrapping_add(carry_add as u16)))?;
        return add_16bit(vm, pipeline, _hv);
    } else {
        let prelim_sum = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Dword(prelim_sum.wrapping_add(carry_add as u32)))?;
        return add_32bit(vm, pipeline, _hv);
    }
}

pub fn sbb_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let carry_sub = if vm.flags.carry{
        1
    }else{
        0
    };
    let prelim_dif = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(prelim_dif.wrapping_sub(carry_sub)))?;    
    return sub_8bit(vm, pipeline, _hv);
}

pub fn sbb_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let carry_sub = if vm.flags.carry{
        1
    }else{
        0
    };
    if pipeline.size_override{
        let prelim_dif = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Word(prelim_dif.wrapping_sub(carry_sub as u16)))?;
        return sub_16bit(vm, pipeline, _hv);
    } else {
        let prelim_dif = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Dword(prelim_dif.wrapping_sub(carry_sub as u32)))?;
        return sub_32bit(vm, pipeline, _hv);
    }
}

pub fn xadd_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    xchg(vm, pipeline, _hv)?;
    return add_8bit(vm, pipeline, _hv);
}

pub fn xadd_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    xchg(vm, pipeline, _hv)?;
    if pipeline.size_override {
        return add_16bit(vm, pipeline, _hv);
    } else {
        return add_32bit(vm, pipeline, _hv);
    }
}

pub fn add_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let adder = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let (result, carry) = base.overflowing_add(adder);
    let (_, overflow) = (base as i8).overflowing_add(adder as i8);
    get_flags(vm, result as u32, adder as u32, base as u32, overflow, carry, AdjustType::Inc, SignType::Byte);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result))?;
    Ok(())
}

pub fn add_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return add_16bit(vm, pipeline, _hv);
    } else {
        return add_32bit(vm, pipeline, _hv);
    }
}

pub fn add_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let adder = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let (result, carry) = base.overflowing_add(adder);
    let (_, overflow) = (base as i16).overflowing_add(adder as i16);
    get_flags(vm, result as u32, adder as u32, base as u32, overflow, carry, AdjustType::Inc, SignType::Word);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result))?;
    Ok(())
}

pub fn add_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let adder = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let (result, carry) = base.overflowing_add(adder);
    let (_, overflow) = (base as i32).overflowing_add(adder as i32);
    get_flags(vm, result as u32, adder as u32, base as u32, overflow, carry, AdjustType::Inc, SignType::Dword);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result))?;
    Ok(())
}

/// The logic function for the `hlt` opcode
pub fn hlt(_vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    Err(VMError::InternalVMStop)
}

pub fn increment_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let (result, overflow) = (base as i8).overflowing_add(1 as i8);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign8(result as u8);
    vm.flags.adjust = (base&0x0F) + (1&0x0F) > 15;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn increment_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return increment_16bit(vm, pipeline, _hv);
    } else {
        return increment_32bit(vm, pipeline, _hv);
    }
}

pub fn increment_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let (result, overflow) = (base as i16).overflowing_add(1 as i16);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign16(result as u16);
    vm.flags.adjust = (base&0x0F) + (1&0x0F) > 15;
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn increment_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let (result, overflow) = (base as i32).overflowing_add(1 as i32);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign32(result as u32);
    vm.flags.adjust = (base&0x0F) + (1&0x0F) > 15;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn sub_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let subt = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let (result, carry) = base.overflowing_sub(subt);
    let (_, overflow) = (base as i8).overflowing_sub(subt as i8);
    get_flags(vm, result as u32, subt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Byte);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result))?;
    Ok(())
}

pub fn sub_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return sub_16bit(vm, pipeline, _hv);
    } else {
        return sub_32bit(vm, pipeline, _hv);
    }
}

pub fn sub_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let subt = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let (result, carry) = base.overflowing_sub(subt);
    let (_, overflow) = (base as i16).overflowing_sub(subt as i16);
    get_flags(vm, result as u32, subt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Word);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result))?;
    Ok(())
}

pub fn sub_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let subt = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let (result, carry) = base.overflowing_sub(subt);
    let (_, overflow) = (base as i32).overflowing_sub(subt as i32);
    get_flags(vm, result as u32, subt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Dword);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result))?;
    Ok(())
}

pub fn decrement_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let (result, overflow) = (base as i8).overflowing_sub(1 as i8);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign8(result as u8);
    vm.flags.adjust = ((base as i32)&0x0F) - ((1 as i32)&0x0F) < 0;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn decrement_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return decrement_16bit(vm, pipeline, _hv);
    } else {
        return decrement_32bit(vm, pipeline, _hv);
    }
}

pub fn decrement_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let (result, overflow) = (base as i16).overflowing_sub(1 as i16);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign16(result as u16);
    vm.flags.adjust = ((base as i32)&0x0F) - ((1 as i32)&0x0F) < 0;
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn decrement_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let (result, overflow) = (base as i32).overflowing_sub(1 as i32);
    vm.flags.overflow = overflow;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    vm.flags.calculate_sign32(result as u32);
    vm.flags.adjust = ((base as i32)&0x0F) - ((1 as i32)&0x0F) < 0;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn cmpxchg_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let accumulator = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let destination = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let source = vm.get_arg(pipeline.args[1].location)?;
    let (result, carry) = accumulator.overflowing_sub(destination);
    let (_, overflow) = (accumulator as i8).overflowing_sub(destination as i8);
    get_flags(vm, result as u32, destination as u32, accumulator as u32, overflow, carry, AdjustType::Dec, SignType::Byte);
    if vm.flags.zero {
        vm.set_arg(pipeline.args[0].location, source)?;
    } else {
        vm.set_reg(Reg8::AL as u8, SizedValue::Byte(destination));
    }
    Ok(())
}

pub fn cmpxchg_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let accumulator = vm.get_reg(Reg16::AX as u8, ValueSize::Word).u16_exact()?;
        let destination = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
        let source = vm.get_arg(pipeline.args[1].location)?;
        let (result, carry) = accumulator.overflowing_sub(destination);
        let (_, overflow) = (accumulator as i16).overflowing_sub(destination as i16);
        get_flags(vm, result as u32, destination as u32, accumulator as u32, overflow, carry, AdjustType::Dec, SignType::Word);
        if vm.flags.zero {
            vm.set_arg(pipeline.args[0].location, source)?;
        } else {
            vm.set_reg(Reg16::AX as u8, SizedValue::Word(destination));
        }
    } else {
        let accumulator = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()?;
        let destination = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
        let source = vm.get_arg(pipeline.args[1].location)?;
        let (result, carry) = accumulator.overflowing_sub(destination);
        let (_, overflow) = (accumulator as i32).overflowing_sub(destination as i32);
        get_flags(vm, result as u32, destination as u32, accumulator as u32, overflow, carry, AdjustType::Dec, SignType::Dword);
        if vm.flags.zero {
            vm.set_arg(pipeline.args[0].location, source)?;
        } else {
            vm.set_reg(Reg32::EAX as u8, SizedValue::Dword(destination));
        }
    }
    Ok(())
}

pub fn cmp_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let cmpt = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let (result, carry) = base.overflowing_sub(cmpt);
    let (_, overflow) = (base as i8).overflowing_sub(cmpt as i8);
    get_flags(vm, result as u32, cmpt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Byte);
    Ok(())
}

pub fn cmp_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return cmp_16bit(vm, pipeline, _hv);
    } else {
        return cmp_32bit(vm, pipeline, _hv);
    }
}

pub fn cmp_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let cmpt = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let (result, carry) = base.overflowing_sub(cmpt);
    let (_, overflow) = (base as i16).overflowing_sub(cmpt as i16);
    get_flags(vm, result as u32, cmpt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Word);
    Ok(())
}

pub fn cmp_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let cmpt = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let (result, carry) = base.overflowing_sub(cmpt);
    let (_, overflow) = (base as i32).overflowing_sub(cmpt as i32);
    get_flags(vm, result as u32, cmpt as u32, base as u32, overflow, carry, AdjustType::Dec, SignType::Dword);
    Ok(())
}

pub fn test_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Byte);
    Ok(())
}

pub fn test_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return and_16bit(vm, pipeline, _hv);
    } else {
        return and_32bit(vm, pipeline, _hv);
    }
}

pub fn test_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Word);
    Ok(())
}

pub fn test_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Dword);
    Ok(())
}


pub fn and_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Byte);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn and_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return and_16bit(vm, pipeline, _hv);
    } else {
        return and_32bit(vm, pipeline, _hv);
    }
}

pub fn and_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Word);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn and_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result = base & mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Dword);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn or_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result = base | mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Byte);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn or_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return or_16bit(vm, pipeline, _hv);
    } else {
        return or_32bit(vm, pipeline, _hv);
    }
}

pub fn or_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result = base | mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Word);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn or_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result = base | mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Dword);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn xor_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u8_exact()?;
    let result = base ^ mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Byte);
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn xor_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return xor_16bit(vm, pipeline, _hv);
    } else {
        return xor_32bit(vm, pipeline, _hv);
    }
}

pub fn xor_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
    let result = base ^ mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Word);
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn xor_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let mask = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    let result = base ^ mask;
    get_flags(vm, result as u32, mask as u32, base as u32, false, false, AdjustType::None, SignType::Dword);
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn not_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    let result = !base;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn not_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return not_16bit(vm, pipeline, _hv);
    } else {
        return not_32bit(vm, pipeline, _hv);
    }
}

pub fn not_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    let result = !base;
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn not_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    let result = !base;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn neg_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    vm.flags.carry = base != 0;
    let (result, overflow) = (base as i8).overflowing_neg();
    vm.flags.calculate_zero(result as u32);
    vm.flags.overflow = overflow;
    vm.set_arg(pipeline.args[0].location, SizedValue::Byte(result as u8))?;
    Ok(())
}

pub fn neg_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError> {
    if pipeline.size_override {
        return neg_16bit(vm, pipeline, _hv);
    } else {
        return neg_32bit(vm, pipeline, _hv);
    }
}

pub fn neg_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u16_exact()?;
    vm.flags.carry = base != 0;
    let (result, overflow) = (base as i16).overflowing_neg();
    vm.flags.calculate_zero(result as u32);
    vm.flags.overflow = overflow;
    vm.set_arg(pipeline.args[0].location, SizedValue::Word(result as u16))?;
    Ok(())
}

pub fn neg_32bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let base = vm.get_arg(pipeline.args[0].location)?.u32_exact()?;
    vm.flags.carry = base != 0;
    let (result, overflow) = (base as i32).overflowing_neg();
    vm.flags.calculate_zero(result as u32);
    vm.flags.overflow = overflow;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(result as u32))?;
    Ok(())
}

pub fn interrupt(vm: &mut VM, pipeline: &Pipeline, hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let num = vm.get_arg(pipeline.args[0].location)?.u8_exact()?;
    hv.interrupt(vm, num)?;
    Ok(())
}

pub fn setcc_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if cc_matches(pipeline.opcode, &vm.flags){
        vm.set_arg(pipeline.args[0].location, SizedValue::Byte(1))?;
    }else{
        vm.set_arg(pipeline.args[0].location, SizedValue::Byte(0))?;
    }
    Ok(())
}

pub fn cmovcc_native(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if cc_matches(pipeline.opcode, &vm.flags){
        vm.set_arg(pipeline.args[0].location, vm.get_arg(pipeline.args[1].location)?)?;
    }
    Ok(())
}

pub fn lea(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let address = vm.get_arg_lea(pipeline.args[1].location)?;
    let value = if pipeline.size_override{
        SizedValue::Word(address as u16)
    }else{
        SizedValue::Dword(address)
    };
    vm.set_arg(pipeline.args[0].location, value)?;
    Ok(())
}

pub fn movzx_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let v = vm.get_arg(pipeline.args[1].location)?.u16_zx()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Word(v))?;
    }else{
        let v = vm.get_arg(pipeline.args[1].location)?.u32_zx()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Dword(v))?;
    }
    Ok(())
}

pub fn movzx_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let v = vm.get_arg(pipeline.args[1].location)?.u32_zx()?;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(v))?;
    Ok(())
}

pub fn movsx_8bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let v = vm.get_arg(pipeline.args[1].location)?.u16_sx()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Word(v))?;
    }else{
        let v = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
        vm.set_arg(pipeline.args[0].location, SizedValue::Dword(v))?;
    }
    Ok(())
}

pub fn movsx_16bit(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let v = vm.get_arg(pipeline.args[1].location)?.u32_sx()?;
    vm.set_arg(pipeline.args[0].location, SizedValue::Dword(v))?;
    Ok(())
}

fn rep_no_flag_opcodes(opcode: u8) -> bool{
    match opcode{
        0xA4 | 0xA5 | //movs
        0xAC | 0xAD | //lods
        0xAA | 0xAB => { //stos
            true
        }
        _ => {
            false
        }
    }
}
fn rep_flag_opcodes(opcode: u8) -> bool{
    match opcode{
        0xA6 | 0xA7 | //cmps
        0xAE | 0xAF => { //scas
            true
        }
        _ => {
            false
        }
    }
}

fn read_regw(vm: &VM, reg: Reg32, size_override: bool) -> u32{
    if size_override{
        vm.regs[reg as usize] & 0x0000FFFF
    }else{
        vm.regs[reg as usize]
    }
}

fn decrement_regw(vm: &mut VM, reg: Reg32, size_override: bool) -> u32{
    if size_override{
        let mut r = (vm.regs[reg as usize] & 0x0000FFFF) as u16;
        r -= 1;
        let write = (vm.regs[reg as usize] & 0xFFFF0000) | (r as u32);
        vm.regs[reg as usize] = write;
        r as u32
    }else{
        vm.regs[reg as usize] -= 1;
        vm.regs[reg as usize]
    } 
}

pub fn repe(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let opcodes = &crate::opcodes::OPCODES;
    let function = opcodes[pipeline.opcode as usize].opcodes[0].function;
    let gas_cost = vm.charger.cost(opcodes[pipeline.opcode as usize].opcodes[0].gas_cost);
    /*      
    while eCX <> 0
        execute string instruction once
        eCX . eCX - 1
    endwhile
    */
    while read_regw(vm, Reg32::ECX, pipeline.size_override) != 0{
        if vm.gas_remaining == 0{
            return Err(VMError::OutOfGas);
        }
        function(vm, pipeline, _hv)?;
        decrement_regw(vm, Reg32::ECX, pipeline.size_override);
        vm.gas_remaining = vm.gas_remaining.saturating_sub(gas_cost);
        if rep_flag_opcodes(pipeline.opcode) {
            if vm.flags.zero == false {
                break;
            }
        }else if rep_no_flag_opcodes(pipeline.opcode) {
            continue
        } else {
            return Err(VMError::InvalidOpcodeEncoding);
        }
    }
    Ok(())
}
pub fn repne(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let opcodes = &crate::opcodes::OPCODES;
    if rep_flag_opcodes(pipeline.opcode){
        let function = opcodes[pipeline.opcode as usize].opcodes[0].function;
        let gas_cost = vm.charger.cost(opcodes[pipeline.opcode as usize].opcodes[0].gas_cost);
        /*      
        while eCX <> 0
            execute string instruction once
            eCX . eCX - 1
        endwhile
        */
        while read_regw(vm, Reg32::ECX, pipeline.size_override) != 0{
            if vm.gas_remaining == 0{
                return Err(VMError::OutOfGas);
            }
            function(vm, pipeline, _hv)?;
            decrement_regw(vm, Reg32::ECX, pipeline.size_override);
            vm.gas_remaining = vm.gas_remaining.saturating_sub(gas_cost);
            if vm.flags.zero{
                break;
            }
        }
    }else{
        //note this prefix can not be used with non-flag using string instructions
        return Err(VMError::InvalidOpcodeEncoding);
    }
    Ok(())
}

pub fn movs_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //[EDI] . [ESI]
    if pipeline.size_override{
        vm.set_mem(vm.reg32(Reg32::EDI), SizedValue::Word(vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Word)?.u16_exact()?))?;
        let d = if vm.flags.direction{
            (-2i32) as u32
        }else{
            2
        };
        //todo DF
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    }else{
        vm.set_mem(vm.reg32(Reg32::EDI), SizedValue::Dword(vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Dword)?.u32_exact()?))?;
        //todo DF
        let d = if vm.flags.direction{
            (-4i32) as u32
        }else{
            4
        };
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    }
    Ok(())
}
pub fn movsb(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //[EDI] . [ESI]
    vm.set_mem(vm.reg32(Reg32::EDI), SizedValue::Byte(vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Byte)?.u8_exact()?))?;
    //todo DF
    let d = if vm.flags.direction{
        (-1i32) as u32
    }else{
        1
    };
    vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    Ok(())
}
pub fn set_direction(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    vm.flags.direction = true;
    Ok(())
}
pub fn clear_direction(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    vm.flags.direction = false;
    Ok(())
}
pub fn cmps_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //[EDI] . [ESI]
    if pipeline.size_override{
        let destination = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Word)?.u16_exact()?;
        let source = vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Word)?.u16_exact()?;
        let (result, carry) = destination.overflowing_sub(source);
        let (_, overflow) = (destination as i16).overflowing_sub(source as i16);
        get_flags(vm, result as u32, source as u32, destination as u32, overflow, carry, AdjustType::Dec, SignType::Word);
        let d = if vm.flags.direction{
            (-2i32) as u32
        }else{
            2
        };
        //todo DF
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    }else{
        let destination = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Dword)?.u32_exact()?;
        let source = vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Dword)?.u32_exact()?;
        let (result, carry) = destination.overflowing_sub(source);
        let (_, overflow) = (destination as i32).overflowing_sub(source as i32);
        get_flags(vm, result as u32, source as u32, destination as u32, overflow, carry, AdjustType::Dec, SignType::Dword);
        //todo DF
        let d = if vm.flags.direction{
            (-4i32) as u32
        }else{
            4
        };
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    }
    Ok(())
}

pub fn cmpsb(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    //[EDI] . [ESI]
    let destination = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Byte)?.u8_exact()?;
    let source = vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Byte)?.u8_exact()?;
    let (result, carry) = destination.overflowing_sub(source);
    let (_, overflow) = (destination as i8).overflowing_sub(source as i8);
    get_flags(vm, result as u32, source as u32, destination as u32, overflow, carry, AdjustType::Dec, SignType::Byte);
    //todo DF
    let d = if vm.flags.direction{
        (-1i32) as u32
    }else{
        1
    };
    vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    
    Ok(())
}

pub fn store_string_byte(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    vm.set_mem(vm.reg32(Reg32::EDI), vm.get_reg(Reg8::AL as u8, ValueSize::Byte))?;
    let d = if vm.flags.direction{
        (-1i32) as u32
    }else{
        1
    };
    vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    Ok(())
}

pub fn load_string_byte(vm: &mut VM, _pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let esi_mem = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Byte)?.u8_exact()?;
    vm.set_reg(Reg8::AL as u8, SizedValue::Byte(esi_mem as u8));
    let d = if vm.flags.direction{
        (-1i32) as u32
    }else{
        1
    };
    vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    Ok(())
}

pub fn scan_string_byte(vm: &mut VM, pipeline: &Pipeline, hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let edi_mem = vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Byte)?.u8_exact()?;
    let al_reg = vm.get_reg(Reg8::AL as u8, ValueSize::Byte).u8_exact()?;
    let (result, carry) = al_reg.overflowing_sub(edi_mem);
    let (_, overflow) = (al_reg as i8).overflowing_sub(edi_mem as i8);
    get_flags(vm, result as u32, edi_mem as u32, al_reg as u32, overflow, carry, AdjustType::Dec, SignType::Byte);
    let d = if vm.flags.direction{
        (-1i32) as u32
    }else{
        1
    };
    vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    Ok(())
}

pub fn store_string_native_word(vm: &mut VM, pipeline: &Pipeline, hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        vm.set_mem(vm.reg32(Reg32::EDI), vm.get_reg(Reg16::AX as u8, ValueSize::Word))?;
        let d = if vm.flags.direction{
            (-2i32) as u32
        }else{
            2
        };
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    } else {
        vm.set_mem(vm.reg32(Reg32::EDI), vm.get_reg(Reg32::EAX as u8, ValueSize::Dword))?;
        let d = if vm.flags.direction{
            (-4i32) as u32
        }else{
            4
        };
        vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    }
    Ok(())
}

pub fn load_string_native_word(vm: &mut VM, pipeline: &Pipeline, _hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    if pipeline.size_override{
        let esi_mem = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Word)?.u16_exact()?;
        vm.set_reg(Reg16::AX as u8, SizedValue::Word(esi_mem));
        let d = if vm.flags.direction{
            (-2i32) as u32
        }else{
            2
        };
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    } else {
        let esi_mem = vm.get_mem(vm.reg32(Reg32::ESI), ValueSize::Dword)?.u32_exact()?;
        vm.set_reg(Reg32::EAX as u8, SizedValue::Dword(esi_mem));
        let d = if vm.flags.direction{
            (-4i32) as u32
        }else{
            4
        };
        vm.set_reg32(Reg32::ESI, vm.reg32(Reg32::ESI).wrapping_add(d));
    }
    Ok(())
}

pub fn scan_string_native_word(vm: &mut VM, pipeline: &Pipeline, hv: &mut dyn Hypervisor) -> Result<(), VMError>{
    let mut d;
    if pipeline.size_override {
        let edi_mem =  vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Word)?.u16_exact()?;
        let ax_reg = vm.get_reg(Reg16::AX as u8, ValueSize::Word).u16_exact()?;
        let (result, carry) = ax_reg.overflowing_sub(edi_mem);
        let (_, overflow) = (ax_reg as i16).overflowing_sub(edi_mem as i16);
        get_flags(vm, result as u32, edi_mem as u32, ax_reg as u32, overflow, carry, AdjustType::Dec, SignType::Word);
        d = if vm.flags.direction{
            (-2i32) as u32
        }else{
            2
        };    
    } else {
        let edi_mem =  vm.get_mem(vm.reg32(Reg32::EDI), ValueSize::Dword)?.u32_exact()?;
        let eax_reg = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()?;
        let (result, carry) = eax_reg.overflowing_sub(edi_mem);
        let (_, overflow) = (eax_reg as i32).overflowing_sub(edi_mem as i32);
        get_flags(vm, result, edi_mem, eax_reg, overflow, carry, AdjustType::Dec, SignType::Dword);
        d = if vm.flags.direction{
            (-4i32) as u32
        }else{
            4
        };
    }
    vm.set_reg32(Reg32::EDI, vm.reg32(Reg32::EDI).wrapping_add(d));
    Ok(())
}

pub enum SignType{
    Byte,
    Word,
    Dword
}

pub enum AdjustType{
    Inc,
    Dec,
    None
}

/// a catch all utility function used by the opcodes to get all flags
/// this should not be used for every single opcode as some have more fine grained nuances
pub fn get_flags(vm: &mut VM, result: u32, source: u32, destination: u32, overflow: bool, carry: bool, adjust: AdjustType, sign: SignType) {
    vm.flags.overflow = overflow;
    vm.flags.carry = carry;
    vm.flags.calculate_zero(result as u32);
    vm.flags.calculate_parity(result as u32);
    match adjust {
        AdjustType::Dec => {
            vm.flags.adjust = ((destination as i32)&0x0F) - ((source as i32)&0x0F) < 0;
        },
        AdjustType::Inc => {
            vm.flags.adjust = (destination&0x0F) + (source&0x0F) > 15;
        },
        AdjustType::None => ()
    }
    match sign {
        SignType::Byte => {
            vm.flags.calculate_sign8(result as u8);
        },
        SignType::Word => {
            vm.flags.calculate_sign16(result as u16);
        },
        SignType::Dword => {
            vm.flags.calculate_sign32(result as u32);
        }
    }
}