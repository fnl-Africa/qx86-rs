use crate::pipeline::*;
use crate::opcodes::*;
use crate::structs::*;
use crate::memory::*;
use crate::flags::*;

#[allow(dead_code)] //remove after design stuff is done

/// The primary controlling VM class holding all state of the machine 
#[derive(Default)]
pub struct VM<'a>{
    pub state: VMState,

    /// The memory of the VM, controlled by the MemorySystem struct
    pub memory: MemorySystem,

    /// set to the EIP value when an error has occurred
    pub error_eip: u32,
    /// The struct which determines how the GasCost tiers resolve into actual numbers
    pub charger: GasCharger,

    pub hypervisor: Option<&'a mut Hypervisor>
}
#[derive(Default, Clone)]
pub struct VMState{
    /// 32bit registers for x86-32
    /// In the order of: EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI
    pub regs: [u32; 8],
    /// The 32bit instruction pointer. This determines where execution of opcodes is happening within memory
    pub eip: u32,
    /// The 32bit instruction pointer. This determines where execution of opcodes is happening within memory
    pub flags: X86Flags,
    /// The amount of gas remaining for execution
    pub gas_remaining: u64
}


pub trait Hypervisor{
    fn interrupt(&mut self, state: &mut VMState, num: u8) -> Result<(), VMError>;
}

/// The gas cost of an operation
#[derive(Debug, Copy, Clone, EnumCount, EnumIter)]
pub enum GasCost{
    /// This operation is free. Used only for nop-like operations
    None,
    /// This operation is of very low cost. Used for operations with very little or simple actual logic
    VeryLow,
    /// This operation is of low cost. Used for operations with some logic, though not a lot of complexity
    Low,
    /// This operation is of moderate cost. Used for somewhat complex operations
    Moderate,
    /// This operation is of high cost. Used for very complex or slow operations
    High,

    //surcharges (not intended to direct use outside of VM)

    /// A surcharge for any branch which can not be predicted. This includes conditional branches and also indirect branches
    ConditionalBranch,
    /// A surcharge for any operation which accesses memory
    MemoryAccess,
    /// A surcharge for every opcode executed within writeable memory space.
    /// Pipelining can not be properly done within writeable memory space due to the risk of the opcodes which are pipelined being changed before execution
    WriteableMemoryExec,
    /// A surcharge for any ModRM argument which must be decoded. This is a relatively complex operation, though can be done fairly efficiently
    ModRMSurcharge
}

impl Default for GasCost{
    fn default() -> GasCost{
        GasCost::Low
    }
}
/// Maps the tiers of GasCost to a numerical value
#[derive(Default, Debug)]
pub struct GasCharger{
    pub costs: [u64; GASCOST_COUNT]
}

impl GasCharger{
    /// Resolve the GasCost value to a numerical value
    pub fn cost(&self, tier: GasCost) -> u64{
        self.costs[tier as usize]
    }

    /// This is a simple default schedule for testing
    /// This is used within integration and benchmarking tests
    pub fn test_schedule() -> GasCharger{
        use GasCost::*;
        let mut g = GasCharger::default();
        g.costs[None as usize] = 0;
        g.costs[VeryLow as usize] = 1;
        g.costs[Low as usize] = 4;
        g.costs[Moderate as usize] = 10;
        g.costs[High as usize] = 20;
        g.costs[ConditionalBranch as usize] = 10;
        g.costs[MemoryAccess as usize] = 1;
        g.costs[WriteableMemoryExec as usize] = 15;
        g.costs[ModRMSurcharge as usize] = 1;
        g
    }
}


/// The 32 bit x86 registers, encoded in the respective order for how the registers are encoded into opcodes and their arguments
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum Reg32{
    EAX = 0,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI
}

/// The 16 bit x86 registers, encoded in the respective order for how the registers are encoded into opcodes and their arguments
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum Reg16{
    AX = 0,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI
}

/// The 8 bit x86 registers, encoded in the respective order for how the registers are encoded into opcodes and their arguments
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum Reg8{
    AL = 0,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH
}

/// All of the types of errors which can be thrown by the qx86 VM
#[derive(PartialEq, Debug, Display)]
#[derive(Copy, Clone)]
pub enum VMError{
    /// Indicates no error
    None,
    /// Indicates this functionality is currently not yet implemented
    /// This will be removed when the qx86 is finished per specifications
    NotYetImplemented,

    //memory errors

    /// Indicates that an access to non-existent memory was attempted
    ReadBadMemory(u32),
    /// Indicates that a write operation to non-existent memory was attempted
    WroteBadMemory(u32),
    /// Indicates that a write operation to read-only memory space was attempted
    WroteReadOnlyMemory(u32),
    /// ???
    ReadUnloadedMemory(u32),

    /// An error thrown by MemorySystem::add_memory which indicates that an address added was not aligned to an 0x10000 byte border
    UnalignedMemoryAddition,
    /// An error thrown by MemorySystem::add_memory which indicates that memory added conflicts with existing memory
    ConflictingMemoryAddition,
    
    //execution error

    /// Indicates that execution of an invalid opcode was attempted. The u8 attached is the primary opcode byte of the opcode
    InvalidOpcode(u8),
    /// This is thrown when writing (ie, using set_arg) a read-only argument is attempted.
    /// This can be triggered for instance by using set_arg on an argument which has a location of ImmediateValue
    WroteUnwriteableArgumnet,

    //decoding error
    /// More bytes were needed for filling the pipeline and decoding opcodes
    /// This can indicate either an overrun into unloaded memory or that there were instructions executed less than 16 bytes before the 0x10000 border of memory
    DecodingOverrun, 
    
    //argument errors

    /// This is thrown when a particular size was expected to be different than the actual size.
    /// This should never be thrown in a completed and bug free VM
    WrongSizeExpectation,
    /// This is thrown when an attempt to convert a particular SizedValue to a particular integer type is attempted and the value will not fit. 
    /// Thsi should never be thrown in a completed and bug free VM
    TooBigSizeExpectation,

    /// This is not an actual error, but rather an "escape" which should stop the VM immediately.
    InternalVMStop,
    /// This indicates that the execution reached the end of its' gas limit
    /// This is not an actual execution error per-se, and resuming afterwards is possible if desired.
    OutOfGas
}


impl<'a> VM<'a>{
    fn calculate_modrm_address(&self, arg: &ArgLocation) -> u32{
        use ArgLocation::*;
        match arg{
            ModRMAddress{offset, reg, size: _} => {
                let o = match offset{
                    Some(x) => *x,
                    Option::None => 0
                };
                let r = match reg{
                    Some(x) => self.state.regs[*x as usize],
                    Option::None => 0
                };
                o.wrapping_add(r)
            },
            SIBAddress{offset, base, scale, index, size: _} => {
                let b = match base{
                    Some(x) => self.state.regs[*x as usize],
                    Option::None => 0
                };
                let ind = match index{
                    Some(x) => self.state.regs[*x as usize],
                    Option::None => 0
                };
                //base + (index * scale) + offset
                let address = b.wrapping_add(ind.wrapping_mul(*scale as u32)).wrapping_add(*offset);
                address
            },
            _ => panic!("This should not be reached")
        }
    }
    /// Resolves an argument location into a SizedValue 
    pub fn get_arg(&self, arg: ArgLocation) -> Result<SizedValue, VMError>{
        use ArgLocation::*;
        Ok(match arg{
            None => SizedValue::None,
            Immediate(v) => v,
            Address(a, s) => {
                self.get_mem(a, s)?
            },
            RegisterValue(r, s) => {
                self.get_reg(r, s)
            },
            RegisterAddress(r, s) => {
                self.get_mem(self.get_reg(r, ValueSize::Dword).u32_exact()?, s)?
            },
            /*
            ModRMAddress16{offset, reg1, reg2, size} => {
                SizedValue::None
            },*/
            ModRMAddress{offset: _, reg: _, size} => {
                self.get_mem(self.calculate_modrm_address(&arg), size)?
            },
            SIBAddress{offset: _, base: _, scale: _, index: _, size} => {
                self.get_mem(self.calculate_modrm_address(&arg), size)?
            }
        })
    }
    /// Resolves an argument location and stores the specified SizedValue in it.
    /// If the specified ArgLocation is of a larger size than the SizedValue, the SizedValue will be zero extended to fit.
    /// If the SizedValue is larger than can fit, then an error will be returned
    pub fn set_arg(&mut self, arg: ArgLocation, v: SizedValue) -> Result<(), VMError>{
        use ArgLocation::*;
        match arg{
            None => (),
            Immediate(_v) => return Err(VMError::WroteUnwriteableArgumnet), //should never happen, this is an implementation error
            Address(a, s) => {
                self.set_mem(a, v.convert_size_zx(s)?)?
            },
            RegisterValue(r, s) => {
                self.set_reg(r, v.convert_size_zx(s)?)
            },
            RegisterAddress(r, s) => {
                self.set_mem(self.get_reg(r, ValueSize::Dword).u32_exact()?, v.convert_size_zx(s)?)?
            },
            ModRMAddress{offset: _, reg: _, size} => {
                let sized = v.convert_size_trunc(size);
                self.set_mem(self.calculate_modrm_address(&arg), sized)?
            },
            SIBAddress{offset: _, base: _, scale: _, index: _, size} => {
                let sized = v.convert_size_trunc(size);
                self.set_mem(self.calculate_modrm_address(&arg), sized)?
            }
        };

        Ok(())
    }
    /// Resolves a numerical register index and ValueSize to a SizedValue indicating the value of that particular register
    pub fn get_reg(&self, reg: u8, size: ValueSize) -> SizedValue{
        use ValueSize::*;
        let r = reg as usize;
        match size{
            ValueSize::None => SizedValue::None,
            Byte => {
                if reg & 0x04 == 0{
                    //access lows, AL, CL, DL, BL
                    SizedValue::Byte((self.state.regs[r] & 0xFF) as u8)
                }else{
                    //access highs, AH, CH, DH, BH
                    SizedValue::Byte(((self.state.regs[r & 0x03] & 0xFF00) >> 8) as u8)
                }
            },
            Word => {
                SizedValue::Word((self.state.regs[r] & 0xFFFF) as u16)
            },
            Dword => {
                SizedValue::Dword(self.state.regs[r])
            }
        }
    }
    /// Resolves a numerical register index and using the size of the SizedValue determiens which 
    /// register to set and sets appropriately to the SizedValue.
    pub fn set_reg(&mut self, reg: u8, value: SizedValue){
        use SizedValue::*;
        let r = reg as usize;
        match value{
            SizedValue::None => (), //could potentially throw an error here?
            Byte(v) => {
                if reg & 0x04 == 0{
                    //access lows, AL, CL, DL, BL
                    self.state.regs[r] = (self.state.regs[r] & 0xFFFFFF00) | (v as u32);
                }else{
                    //access highs, AH, CH, DH, BH
                    self.state.regs[r & 0x03] = (self.state.regs[r & 0x03] & 0xFFFF00FF) | ((v as u32) << 8);
                }
            },
            Word(v) => {
                self.state.regs[r] = (self.state.regs[r] & 0xFFFF0000) | (v as u32);
            },
            Dword(v) => {
                self.state.regs[r] = v;
            }
        }
    }
    /// Retreives a SizedValue from VM memory which matches the specified ValueSize
    pub fn get_mem(&self, address: u32, size: ValueSize) -> Result<SizedValue, VMError>{
        use ValueSize::*;
        match size{
            None => Ok(SizedValue::None),
            Byte => {
                Ok(SizedValue::Byte(self.memory.get_u8(address)?))
            },
            Word => {
                Ok(SizedValue::Word(self.memory.get_u16(address)?))
            },
            Dword => {
                Ok(SizedValue::Dword(self.memory.get_u32(address)?))
            },
        }
    }
    /// Sets an area in VM memory to the specified SizedValue
    pub fn set_mem(&mut self, address: u32, value: SizedValue) -> Result<(), VMError>{
        use SizedValue::*;
        if address & 0x80000000 == 0{
            //in read-only memory
            return Err(VMError::WroteReadOnlyMemory(address));
        }
        match value{
            None => (),
            Byte(v) => {
                self.memory.set_u8(address, v)?;
            },
            Word(v) => {
                self.memory.set_u16(address, v)?;
            },
            Dword(v) => {
                self.memory.set_u32(address, v)?;
            }
        };
        Ok(())
    }

    /// This will execute one "cycle" of the VM
    /// A cycle includes filling the pipeline, executing the filled pipeline, and then handling any errors present
    /// Will return a result of true if an InternalVMStop was received, otherwise will return false
    fn cycle(&mut self, pipeline: &mut [Pipeline]) -> Result<bool, VMError>{
        fill_pipeline(self, &OPCODES[0..], pipeline)?;
        //manually unroll loop later if needed?
        for n in 0..pipeline.len() {
            let p = &pipeline[n];
            let (_, negative_gas) = self.state.gas_remaining.overflowing_sub(p.gas_cost);
            self.state.gas_remaining = self.state.gas_remaining.saturating_sub(p.gas_cost);

            //the pipeline will not be filled beyond out of gas, so no worries about inconsistent errored state here
            //micro optimization note: removing this branch results in ~1% performance increase in naive tests
            //but changes the result of EIP to be incorrect.. Decide later if inaccurate EIP is worth that 1%
            if negative_gas {
                return Err(VMError::OutOfGas);
            }
            //errors[n] = (p.function)(self, p);
            let r = (p.function)(self,p);
            if r.is_err(){
                if r.err().unwrap() == VMError::InternalVMStop{
                    return Ok(true);
                }else{
                    self.error_eip = self.state.eip;
                    return Err(r.err().unwrap());
                }
            }
            self.state.eip = self.state.eip.wrapping_add(p.eip_size as u32);
        }
        return Ok(false);

    }
    /// Executes the VM either until there is no remaining gas, an error occurs, or the `hlt` instruction is executed
    pub fn execute(&mut self) -> Result<bool, VMError>{
        //todo: gas handling
        let mut pipeline = vec![];
        pipeline.resize(PIPELINE_SIZE, Pipeline::default());
        loop{
            if self.cycle(&mut pipeline)? {
                return Ok(true);
            }
        }
    }
    /// Helper function to simplify copying a set of data into VM memory
    pub fn copy_into_memory(&mut self, address: u32, data: &[u8]) -> Result<(), VMError>{
        let m = self.memory.get_mut_sized_memory(address, data.len() as u32)?;
        m[0..data.len()].copy_from_slice(data);
        Ok(())
    }
    pub fn reg8(&self, r: Reg8) -> u8{
        self.get_reg(r as u8, ValueSize::Byte).u8_exact().unwrap()
    }
    pub fn reg16(&self, r: Reg16) -> u16{
        self.get_reg(r as u8, ValueSize::Word).u16_exact().unwrap()
    }
    pub fn reg32(&self, r: Reg32) -> u32{
        self.get_reg(r as u8, ValueSize::Dword).u32_exact().unwrap()
    }
    pub fn set_reg8(&mut self, r: Reg8, v: u8){
        self.set_reg(r as u8, SizedValue::Byte(v));
    }
    pub fn set_reg16(&mut self, r: Reg8, v: u16){
        self.set_reg(r as u8, SizedValue::Word(v));
    }
    pub fn set_reg32(&mut self, r: Reg8, v: u32){
        self.set_reg(r as u8, SizedValue::Dword(v));
    }
}

/// The size of the pipeline is fixed in order to avoid unnecessary allocation in the main loop of the VM, 
/// as well as to give Rust additional information for optimizations.
/// This constant determines the size of that pipeline.
/// This does not affect the actual behavior or results of the VM, and only causes a performance change.
/// 
/// Performance tuning guidance:
/// 
/// Too small and predictable runs of instructions become slower than needed by requiring
/// unnecessary (host CPU) cache switching between the execution logic and decoding logic of VM instructions
/// 
/// Too large and the cost of unpredictable instructions becomes greater due to 
/// requiring execution of a greater number of simple, but still time consuming "nop"s 
/// 
/// Somewhere in between 10 and 30 seems to be about right judging from reverse engineered programs, but ultimately
/// it will require extensive benchmarking to be completely sure about the final value.
const PIPELINE_SIZE:usize = 16;

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_memory(){
        let mut m = MemorySystem::default();
        let bytes = m.add_memory(0x10000, 0x100).unwrap();
        bytes[0x10] = 0x12;
        assert!(m.get_memory(0x10010).unwrap()[0] == 0x12);
        let mb = m.get_mut_memory(0x10020).unwrap();
        mb[0] = 0x34;
        assert!(m.get_memory(0x10020).unwrap()[0] == 0x34);
        assert!(m.section_exists(0x10000));
        assert!(!m.section_exists(0x20000));
    }
    #[test]
    fn test_memory_ints(){
        let mut m = MemorySystem::default();
        let area = 0x100000;
        let bytes = m.add_memory(area, 0x100).unwrap();
        bytes[0x10] = 0x11; 
        bytes[0x11] = 0x22;
        bytes[0x12] = 0x33;
        bytes[0x13] = 0x44;
        assert!(m.get_u8(area + 0x10).unwrap() == 0x11);
        assert!(m.get_u8(area + 0x11).unwrap() == 0x22);
        assert!(m.get_u16(area + 0x10).unwrap() == 0x2211);
        assert!(m.get_u32(area + 0x10).unwrap() == 0x44332211);
        m.set_u32(area + 0x20, 0xAABBCCDD).unwrap();
        m.set_u16(area + 0x30, 0x1234).unwrap();
        let bytes = m.get_memory(area).unwrap(); //reassign bytes to avoid borrowing error, can't overlap with previous set operations
        assert!(bytes[0x20] == 0xDD);
        assert!(bytes[0x21] == 0xCC);
        assert!(bytes[0x22] == 0xBB);
        assert!(bytes[0x23] == 0xAA);
    }
    #[test]
    fn test_memory_failures(){
        let mut m = MemorySystem::default();
        let _bytes = m.add_memory(0x10000, 0x100).unwrap();
        assert!(m.add_memory(0x10000, 0x100) == Err(VMError::ConflictingMemoryAddition));
        assert!(m.add_memory(0x100FF, 0x100) == Err(VMError::UnalignedMemoryAddition));
        assert!(m.get_memory(0x10200) == Err(VMError::ReadBadMemory(0x10200)));
        assert!(m.get_mut_memory(0x10100) == Err(VMError::WroteBadMemory(0x10100)));
    }

    #[test]
    fn test_register_access(){
        let mut vm = VM::default();
        vm.state.regs[2] = 0x11223344; // EDX
        vm.state.regs[4] = 0xFFEEDDBB; // ESP
        assert!(vm.get_reg(2, ValueSize::Dword) == SizedValue::Dword(0x11223344));
        assert!(vm.get_reg(4, ValueSize::Dword) == SizedValue::Dword(0xFFEEDDBB));
        assert!(vm.get_reg(2, ValueSize::Word) == SizedValue::Word(0x3344));
        assert!(vm.get_reg(4, ValueSize::Word) == SizedValue::Word(0xDDBB));
        assert!(vm.get_reg(2, ValueSize::Byte) == SizedValue::Byte(0x44)); //DL
        assert!(vm.get_reg(6, ValueSize::Byte) == SizedValue::Byte(0x33)); //DH
    }
    #[test]
    fn test_register_writes(){
        use SizedValue::*;
        let mut vm = VM::default();
        vm.state.regs[2] = 0x11223344; // EDX
        vm.state.regs[4] = 0xFFEEDDBB; // ESP
        vm.set_reg(2, Dword(0xAABBCCDD));
        assert!(vm.get_reg(2, ValueSize::Dword) == SizedValue::Dword(0xAABBCCDD));
        vm.set_reg(4, Word(0x1122));
        assert!(vm.state.regs[4] == 0xFFEE1122);
        vm.set_reg(2, Byte(0x55)); //DL
        assert!(vm.state.regs[2] == 0xAABBCC55);
        vm.set_reg(6, Byte(0x66)); //DH
        assert_eq!(vm.state.regs[2], 0xAABB6655);
    }

    #[test]
    fn test_get_arg(){
        use SizedValue::*;
        let mut vm = VM::default();
        let area = 0x77660000;
        vm.memory.add_memory(area, 0x100).unwrap();
        vm.memory.set_u32(area + 10, 0x11223344).unwrap();
        vm.state.regs[0] = area + 12; //eax
        
        let arg = ArgLocation::Immediate(Word(0x9911));
        assert!(vm.get_arg(arg).unwrap() == Word(0x9911));
        let arg = ArgLocation::Address(area + 10, ValueSize::Dword);
        assert!(vm.get_arg(arg).unwrap() == Dword(0x11223344));
        let arg = ArgLocation::RegisterAddress(0, ValueSize::Word);
        assert!(vm.get_arg(arg).unwrap() == Word(0x1122));
        let arg = ArgLocation::RegisterValue(0, ValueSize::Dword);
        assert!(vm.get_arg(arg).unwrap() == Dword(area + 12));
    }

    #[test]
    fn test_set_arg(){
        use SizedValue::*;
        let mut vm = VM::default();
        let area = 0x87660000; //make sure top bit of memory area is set so it's writeable
        vm.memory.add_memory(area, 0x100).unwrap();
        vm.memory.set_u32(area + 10, 0x11223344).unwrap();
        vm.state.regs[0] = area + 12; //eax
        

        let arg = ArgLocation::Immediate(Word(0x9911));
        assert!(vm.set_arg(arg, SizedValue::Byte(0x11)).is_err());

        let arg = ArgLocation::Address(area + 10, ValueSize::Dword);
        vm.set_arg(arg, SizedValue::Dword(0xAABBCCDD)).unwrap();
        assert!(vm.memory.get_u32(area + 10).unwrap() == 0xAABBCCDD);

        let arg = ArgLocation::RegisterAddress(0, ValueSize::Word);
        vm.set_arg(arg, SizedValue::Word(0x1122)).unwrap();
        assert!(vm.memory.get_u32(area + 10).unwrap() == 0x1122CCDD);
        //test that it fails for too large of values
        assert!(vm.set_arg(arg, SizedValue::Dword(0xABCDEF01)).is_err());
        //and that memory is unmodified afterwards
        assert!(vm.memory.get_u32(area + 10).unwrap() == 0x1122CCDD);

        vm.set_arg(arg, SizedValue::Byte(0x11)).unwrap();
        assert!(vm.memory.get_u32(area + 10).unwrap() == 0x0011CCDD);

        let arg = ArgLocation::RegisterValue(1, ValueSize::Dword);
        vm.set_arg(arg, SizedValue::Dword(0x99887766)).unwrap();
        assert!(vm.state.regs[1] == 0x99887766);
    }
    #[test]
    fn test_set_arg_readonly(){
        let mut vm = VM::default();
        let area = 0x07660000;
        vm.memory.add_memory(area, 0x100).unwrap();
        vm.memory.set_u32(area + 10, 0x11223344).unwrap();
        vm.state.regs[0] = area + 12; //eax
        
        let arg = ArgLocation::Address(area + 10, ValueSize::Dword);
        //ensure it errors
        assert!(vm.set_arg(arg, SizedValue::Dword(0xAABBCCDD)).is_err());
        //and that memory is unchanged
        assert!(vm.memory.get_u32(area + 10).unwrap() == 0x11223344);
    }
    #[test]
    fn test_modrm_address_calculation(){
        let mut vm = VM::default();
        let a = ArgLocation::ModRMAddress{
            offset: None,
            reg: Some(Reg32::EBX as u8),
            size: ValueSize::Byte
        };
        vm.state.regs[Reg32::EBX as usize] = 0x88112233;
        assert_eq!(vm.calculate_modrm_address(&a), 0x88112233);
        let a = ArgLocation::ModRMAddress{
            offset: Some(0x11223344),
            reg: Some(Reg32::EBX as u8),
            size: ValueSize::Byte
        };
        assert_eq!(vm.calculate_modrm_address(&a), 0x99335577);
        let a = ArgLocation::ModRMAddress{
            offset: Some(0x11223344),
            reg: None,
            size: ValueSize::Byte
        };
        assert_eq!(vm.calculate_modrm_address(&a), 0x11223344);
    }
    #[test]
    fn test_sib_address_calculation(){
        let mut vm = VM::default();
        let a = ArgLocation::SIBAddress{
            offset: 1,
            base: Some(Reg32::EBX as u8),
            scale: 2,
            index: Some(Reg32::EDI as u8),
            size: ValueSize::Byte
        };
        vm.state.regs[Reg32::EBX as usize] = 0x11223344;
        vm.state.regs[Reg32::EDI as usize] = 0xFFEEDDCC;
        //(ffeeddcc * 2) + 11223344 + 1
        //(ffddbb98) + 11223344 + 1
        //10ffeddc + 1
        //10ffeddd
        assert_eq!(vm.calculate_modrm_address(&a), 0x10FFEEDD);

    }
}
