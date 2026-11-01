/// An instruction format
///
/// Every opcode has a corresponding instruction format
/// which is represented by both the `InstructionFormat`
/// and the `InstructionData` enums.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InstructionFormat {
    /// AtomicCas(imms=(flags: ir::MemFlags), vals=3, blocks=0)
    AtomicCas, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// AtomicRmw(imms=(flags: ir::MemFlags, op: ir::AtomicRmwOp), vals=2, blocks=0)
    AtomicRmw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Binary(imms=(), vals=2, blocks=0)
    Binary, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// BinaryImm64(imms=(imm: ir::immediates::Imm64), vals=1, blocks=0)
    BinaryImm64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// BinaryImm8(imms=(imm: ir::immediates::Uimm8), vals=1, blocks=0)
    BinaryImm8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// BranchTable(imms=(table: ir::JumpTable), vals=1, blocks=0)
    BranchTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Brif(imms=(), vals=1, blocks=2)
    Brif, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Call(imms=(func_ref: ir::FuncRef), vals=0, blocks=0)
    Call, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// CallIndirect(imms=(sig_ref: ir::SigRef), vals=1, blocks=0)
    CallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// CondTrap(imms=(code: ir::TrapCode), vals=1, blocks=0)
    CondTrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// DynamicStackLoad(imms=(dynamic_stack_slot: ir::DynamicStackSlot), vals=0, blocks=0)
    DynamicStackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// DynamicStackStore(imms=(dynamic_stack_slot: ir::DynamicStackSlot), vals=1, blocks=0)
    DynamicStackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// FloatCompare(imms=(cond: ir::condcodes::FloatCC), vals=2, blocks=0)
    FloatCompare, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// FuncAddr(imms=(func_ref: ir::FuncRef), vals=0, blocks=0)
    FuncAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// IntAddTrap(imms=(code: ir::TrapCode), vals=2, blocks=0)
    IntAddTrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// IntCompare(imms=(cond: ir::condcodes::IntCC), vals=2, blocks=0)
    IntCompare, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// IntCompareImm(imms=(cond: ir::condcodes::IntCC, imm: ir::immediates::Imm64), vals=1, blocks=0)
    IntCompareImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Jump(imms=(), vals=0, blocks=1)
    Jump, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Load(imms=(flags: ir::MemFlags, offset: ir::immediates::Offset32), vals=1, blocks=0)
    Load, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// LoadNoOffset(imms=(flags: ir::MemFlags), vals=1, blocks=0)
    LoadNoOffset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// MultiAry(imms=(), vals=0, blocks=0)
    MultiAry, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// NullAry(imms=(), vals=0, blocks=0)
    NullAry, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Shuffle(imms=(imm: ir::Immediate), vals=2, blocks=0)
    Shuffle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// StackLoad(imms=(stack_slot: ir::StackSlot, offset: ir::immediates::Offset32), vals=0, blocks=0)
    StackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// StackStore(imms=(stack_slot: ir::StackSlot, offset: ir::immediates::Offset32), vals=1, blocks=0)
    StackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Store(imms=(flags: ir::MemFlags, offset: ir::immediates::Offset32), vals=2, blocks=0)
    Store, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// StoreNoOffset(imms=(flags: ir::MemFlags), vals=2, blocks=0)
    StoreNoOffset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Ternary(imms=(), vals=3, blocks=0)
    Ternary, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// TernaryImm8(imms=(imm: ir::immediates::Uimm8), vals=2, blocks=0)
    TernaryImm8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Trap(imms=(code: ir::TrapCode), vals=0, blocks=0)
    Trap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// TryCall(imms=(func_ref: ir::FuncRef, exception: ir::ExceptionTable), vals=0, blocks=0)
    TryCall, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// TryCallIndirect(imms=(exception: ir::ExceptionTable), vals=1, blocks=0)
    TryCallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// Unary(imms=(), vals=1, blocks=0)
    Unary, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryConst(imms=(constant_handle: ir::Constant), vals=0, blocks=0)
    UnaryConst, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryGlobalValue(imms=(global_value: ir::GlobalValue), vals=0, blocks=0)
    UnaryGlobalValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryIeee16(imms=(imm: ir::immediates::Ieee16), vals=0, blocks=0)
    UnaryIeee16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryIeee32(imms=(imm: ir::immediates::Ieee32), vals=0, blocks=0)
    UnaryIeee32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryIeee64(imms=(imm: ir::immediates::Ieee64), vals=0, blocks=0)
    UnaryIeee64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
    /// UnaryImm(imms=(imm: ir::immediates::Imm64), vals=0, blocks=0)
    UnaryImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:32
}

impl<'a> From<&'a InstructionData> for InstructionFormat {
    fn from(inst: &'a InstructionData) -> Self {
        match *inst {
            InstructionData::AtomicCas { .. } => {
                Self::AtomicCas
            }
            InstructionData::AtomicRmw { .. } => {
                Self::AtomicRmw
            }
            InstructionData::Binary { .. } => {
                Self::Binary
            }
            InstructionData::BinaryImm64 { .. } => {
                Self::BinaryImm64
            }
            InstructionData::BinaryImm8 { .. } => {
                Self::BinaryImm8
            }
            InstructionData::BranchTable { .. } => {
                Self::BranchTable
            }
            InstructionData::Brif { .. } => {
                Self::Brif
            }
            InstructionData::Call { .. } => {
                Self::Call
            }
            InstructionData::CallIndirect { .. } => {
                Self::CallIndirect
            }
            InstructionData::CondTrap { .. } => {
                Self::CondTrap
            }
            InstructionData::DynamicStackLoad { .. } => {
                Self::DynamicStackLoad
            }
            InstructionData::DynamicStackStore { .. } => {
                Self::DynamicStackStore
            }
            InstructionData::FloatCompare { .. } => {
                Self::FloatCompare
            }
            InstructionData::FuncAddr { .. } => {
                Self::FuncAddr
            }
            InstructionData::IntAddTrap { .. } => {
                Self::IntAddTrap
            }
            InstructionData::IntCompare { .. } => {
                Self::IntCompare
            }
            InstructionData::IntCompareImm { .. } => {
                Self::IntCompareImm
            }
            InstructionData::Jump { .. } => {
                Self::Jump
            }
            InstructionData::Load { .. } => {
                Self::Load
            }
            InstructionData::LoadNoOffset { .. } => {
                Self::LoadNoOffset
            }
            InstructionData::MultiAry { .. } => {
                Self::MultiAry
            }
            InstructionData::NullAry { .. } => {
                Self::NullAry
            }
            InstructionData::Shuffle { .. } => {
                Self::Shuffle
            }
            InstructionData::StackLoad { .. } => {
                Self::StackLoad
            }
            InstructionData::StackStore { .. } => {
                Self::StackStore
            }
            InstructionData::Store { .. } => {
                Self::Store
            }
            InstructionData::StoreNoOffset { .. } => {
                Self::StoreNoOffset
            }
            InstructionData::Ternary { .. } => {
                Self::Ternary
            }
            InstructionData::TernaryImm8 { .. } => {
                Self::TernaryImm8
            }
            InstructionData::Trap { .. } => {
                Self::Trap
            }
            InstructionData::TryCall { .. } => {
                Self::TryCall
            }
            InstructionData::TryCallIndirect { .. } => {
                Self::TryCallIndirect
            }
            InstructionData::Unary { .. } => {
                Self::Unary
            }
            InstructionData::UnaryConst { .. } => {
                Self::UnaryConst
            }
            InstructionData::UnaryGlobalValue { .. } => {
                Self::UnaryGlobalValue
            }
            InstructionData::UnaryIeee16 { .. } => {
                Self::UnaryIeee16
            }
            InstructionData::UnaryIeee32 { .. } => {
                Self::UnaryIeee32
            }
            InstructionData::UnaryIeee64 { .. } => {
                Self::UnaryIeee64
            }
            InstructionData::UnaryImm { .. } => {
                Self::UnaryImm
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[allow(missing_docs, reason = "generated code")]
pub enum InstructionData {
    AtomicCas {
        opcode: Opcode,
        args: [Value; 3], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    AtomicRmw {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        op: ir::AtomicRmwOp, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Binary {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    BinaryImm64 {
        opcode: Opcode,
        arg: Value,
        imm: ir::immediates::Imm64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    BinaryImm8 {
        opcode: Opcode,
        arg: Value,
        imm: ir::immediates::Uimm8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    BranchTable {
        opcode: Opcode,
        arg: Value,
        table: ir::JumpTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Brif {
        opcode: Opcode,
        arg: Value,
        blocks: [ir::BlockCall; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:82
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Call {
        opcode: Opcode,
        args: ValueList,
        func_ref: ir::FuncRef, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    CallIndirect {
        opcode: Opcode,
        args: ValueList,
        sig_ref: ir::SigRef, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    CondTrap {
        opcode: Opcode,
        arg: Value,
        code: ir::TrapCode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    DynamicStackLoad {
        opcode: Opcode,
        dynamic_stack_slot: ir::DynamicStackSlot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    DynamicStackStore {
        opcode: Opcode,
        arg: Value,
        dynamic_stack_slot: ir::DynamicStackSlot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    FloatCompare {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        cond: ir::condcodes::FloatCC, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    FuncAddr {
        opcode: Opcode,
        func_ref: ir::FuncRef, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    IntAddTrap {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        code: ir::TrapCode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    IntCompare {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        cond: ir::condcodes::IntCC, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    IntCompareImm {
        opcode: Opcode,
        arg: Value,
        cond: ir::condcodes::IntCC, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        imm: ir::immediates::Imm64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Jump {
        opcode: Opcode,
        destination: ir::BlockCall,
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Load {
        opcode: Opcode,
        arg: Value,
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        offset: ir::immediates::Offset32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    LoadNoOffset {
        opcode: Opcode,
        arg: Value,
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    MultiAry {
        opcode: Opcode,
        args: ValueList,
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    NullAry {
        opcode: Opcode,
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Shuffle {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        imm: ir::Immediate, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    StackLoad {
        opcode: Opcode,
        stack_slot: ir::StackSlot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        offset: ir::immediates::Offset32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    StackStore {
        opcode: Opcode,
        arg: Value,
        stack_slot: ir::StackSlot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        offset: ir::immediates::Offset32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Store {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        offset: ir::immediates::Offset32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    StoreNoOffset {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        flags: ir::MemFlags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Ternary {
        opcode: Opcode,
        args: [Value; 3], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    TernaryImm8 {
        opcode: Opcode,
        args: [Value; 2], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:76
        imm: ir::immediates::Uimm8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Trap {
        opcode: Opcode,
        code: ir::TrapCode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    TryCall {
        opcode: Opcode,
        args: ValueList,
        func_ref: ir::FuncRef, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
        exception: ir::ExceptionTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    TryCallIndirect {
        opcode: Opcode,
        args: ValueList,
        exception: ir::ExceptionTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    Unary {
        opcode: Opcode,
        arg: Value,
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryConst {
        opcode: Opcode,
        constant_handle: ir::Constant, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryGlobalValue {
        opcode: Opcode,
        global_value: ir::GlobalValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryIeee16 {
        opcode: Opcode,
        imm: ir::immediates::Ieee16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryIeee32 {
        opcode: Opcode,
        imm: ir::immediates::Ieee32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryIeee64 {
        opcode: Opcode,
        imm: ir::immediates::Ieee64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
    UnaryImm {
        opcode: Opcode,
        imm: ir::immediates::Imm64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:91
    }
    , // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:94
}

impl InstructionData {
    /// Get the opcode of this instruction.
    pub fn opcode(&self) -> Opcode {
        match *self {
            Self::AtomicCas { opcode, .. } |
            Self::AtomicRmw { opcode, .. } |
            Self::Binary { opcode, .. } |
            Self::BinaryImm64 { opcode, .. } |
            Self::BinaryImm8 { opcode, .. } |
            Self::BranchTable { opcode, .. } |
            Self::Brif { opcode, .. } |
            Self::Call { opcode, .. } |
            Self::CallIndirect { opcode, .. } |
            Self::CondTrap { opcode, .. } |
            Self::DynamicStackLoad { opcode, .. } |
            Self::DynamicStackStore { opcode, .. } |
            Self::FloatCompare { opcode, .. } |
            Self::FuncAddr { opcode, .. } |
            Self::IntAddTrap { opcode, .. } |
            Self::IntCompare { opcode, .. } |
            Self::IntCompareImm { opcode, .. } |
            Self::Jump { opcode, .. } |
            Self::Load { opcode, .. } |
            Self::LoadNoOffset { opcode, .. } |
            Self::MultiAry { opcode, .. } |
            Self::NullAry { opcode, .. } |
            Self::Shuffle { opcode, .. } |
            Self::StackLoad { opcode, .. } |
            Self::StackStore { opcode, .. } |
            Self::Store { opcode, .. } |
            Self::StoreNoOffset { opcode, .. } |
            Self::Ternary { opcode, .. } |
            Self::TernaryImm8 { opcode, .. } |
            Self::Trap { opcode, .. } |
            Self::TryCall { opcode, .. } |
            Self::TryCallIndirect { opcode, .. } |
            Self::Unary { opcode, .. } |
            Self::UnaryConst { opcode, .. } |
            Self::UnaryGlobalValue { opcode, .. } |
            Self::UnaryIeee16 { opcode, .. } |
            Self::UnaryIeee32 { opcode, .. } |
            Self::UnaryIeee64 { opcode, .. } |
            Self::UnaryImm { opcode, .. } => {
                opcode
            }
        }
    }

    /// Get the controlling type variable operand.
    pub fn typevar_operand(&self, pool: &ir::ValueListPool) -> Option<Value> {
        match *self {
            Self::Call { .. } |
            Self::DynamicStackLoad { .. } |
            Self::FuncAddr { .. } |
            Self::Jump { .. } |
            Self::MultiAry { .. } |
            Self::NullAry { .. } |
            Self::StackLoad { .. } |
            Self::Trap { .. } |
            Self::TryCall { .. } |
            Self::UnaryConst { .. } |
            Self::UnaryGlobalValue { .. } |
            Self::UnaryIeee16 { .. } |
            Self::UnaryIeee32 { .. } |
            Self::UnaryIeee64 { .. } |
            Self::UnaryImm { .. } => {
                None
            }
            Self::BinaryImm64 { arg, .. } |
            Self::BinaryImm8 { arg, .. } |
            Self::BranchTable { arg, .. } |
            Self::Brif { arg, .. } |
            Self::CondTrap { arg, .. } |
            Self::DynamicStackStore { arg, .. } |
            Self::IntCompareImm { arg, .. } |
            Self::Load { arg, .. } |
            Self::LoadNoOffset { arg, .. } |
            Self::StackStore { arg, .. } |
            Self::Unary { arg, .. } => {
                Some(arg)
            }
            Self::AtomicRmw { args: ref args_arity2, .. } |
            Self::Binary { args: ref args_arity2, .. } |
            Self::FloatCompare { args: ref args_arity2, .. } |
            Self::IntAddTrap { args: ref args_arity2, .. } |
            Self::IntCompare { args: ref args_arity2, .. } |
            Self::Shuffle { args: ref args_arity2, .. } |
            Self::Store { args: ref args_arity2, .. } |
            Self::StoreNoOffset { args: ref args_arity2, .. } |
            Self::TernaryImm8 { args: ref args_arity2, .. } => {
                Some(args_arity2[0])
            }
            Self::Ternary { args: ref args_arity3, .. } => {
                Some(args_arity3[1])
            }
            Self::AtomicCas { args: ref args_arity3, .. } => {
                Some(args_arity3[2])
            }
            Self::CallIndirect { ref args, .. } |
            Self::TryCallIndirect { ref args, .. } => {
                args.get(0, pool)
            }
        }
    }

    /// Get the value arguments to this instruction.
    pub fn arguments<'a>(&'a self, pool: &'a ir::ValueListPool) -> &'a [Value] {
        match *self {
            Self::DynamicStackLoad { .. } |
            Self::FuncAddr { .. } |
            Self::Jump { .. } |
            Self::NullAry { .. } |
            Self::StackLoad { .. } |
            Self::Trap { .. } |
            Self::UnaryConst { .. } |
            Self::UnaryGlobalValue { .. } |
            Self::UnaryIeee16 { .. } |
            Self::UnaryIeee32 { .. } |
            Self::UnaryIeee64 { .. } |
            Self::UnaryImm { .. } => {
                &[]
            }
            Self::AtomicRmw { args: ref args_arity2, .. } |
            Self::Binary { args: ref args_arity2, .. } |
            Self::FloatCompare { args: ref args_arity2, .. } |
            Self::IntAddTrap { args: ref args_arity2, .. } |
            Self::IntCompare { args: ref args_arity2, .. } |
            Self::Shuffle { args: ref args_arity2, .. } |
            Self::Store { args: ref args_arity2, .. } |
            Self::StoreNoOffset { args: ref args_arity2, .. } |
            Self::TernaryImm8 { args: ref args_arity2, .. } => {
                args_arity2
            }
            Self::AtomicCas { args: ref args_arity3, .. } |
            Self::Ternary { args: ref args_arity3, .. } => {
                args_arity3
            }
            Self::BinaryImm64 { ref arg, .. } |
            Self::BinaryImm8 { ref arg, .. } |
            Self::BranchTable { ref arg, .. } |
            Self::Brif { ref arg, .. } |
            Self::CondTrap { ref arg, .. } |
            Self::DynamicStackStore { ref arg, .. } |
            Self::IntCompareImm { ref arg, .. } |
            Self::Load { ref arg, .. } |
            Self::LoadNoOffset { ref arg, .. } |
            Self::StackStore { ref arg, .. } |
            Self::Unary { ref arg, .. } => {
                core::slice::from_ref(arg)
            }
            Self::Call { ref args, .. } |
            Self::CallIndirect { ref args, .. } |
            Self::MultiAry { ref args, .. } |
            Self::TryCall { ref args, .. } |
            Self::TryCallIndirect { ref args, .. } => {
                args.as_slice(pool)
            }
        }
    }

    /// Get mutable references to the value arguments to this
    /// instruction.
    pub fn arguments_mut<'a>(&'a mut self, pool: &'a mut ir::ValueListPool) -> &'a mut [Value] {
        match *self {
            Self::DynamicStackLoad { .. } |
            Self::FuncAddr { .. } |
            Self::Jump { .. } |
            Self::NullAry { .. } |
            Self::StackLoad { .. } |
            Self::Trap { .. } |
            Self::UnaryConst { .. } |
            Self::UnaryGlobalValue { .. } |
            Self::UnaryIeee16 { .. } |
            Self::UnaryIeee32 { .. } |
            Self::UnaryIeee64 { .. } |
            Self::UnaryImm { .. } => {
                &mut []
            }
            Self::AtomicRmw { args: ref mut args_arity2, .. } |
            Self::Binary { args: ref mut args_arity2, .. } |
            Self::FloatCompare { args: ref mut args_arity2, .. } |
            Self::IntAddTrap { args: ref mut args_arity2, .. } |
            Self::IntCompare { args: ref mut args_arity2, .. } |
            Self::Shuffle { args: ref mut args_arity2, .. } |
            Self::Store { args: ref mut args_arity2, .. } |
            Self::StoreNoOffset { args: ref mut args_arity2, .. } |
            Self::TernaryImm8 { args: ref mut args_arity2, .. } => {
                args_arity2
            }
            Self::AtomicCas { args: ref mut args_arity3, .. } |
            Self::Ternary { args: ref mut args_arity3, .. } => {
                args_arity3
            }
            Self::BinaryImm64 { ref mut arg, .. } |
            Self::BinaryImm8 { ref mut arg, .. } |
            Self::BranchTable { ref mut arg, .. } |
            Self::Brif { ref mut arg, .. } |
            Self::CondTrap { ref mut arg, .. } |
            Self::DynamicStackStore { ref mut arg, .. } |
            Self::IntCompareImm { ref mut arg, .. } |
            Self::Load { ref mut arg, .. } |
            Self::LoadNoOffset { ref mut arg, .. } |
            Self::StackStore { ref mut arg, .. } |
            Self::Unary { ref mut arg, .. } => {
                core::slice::from_mut(arg)
            }
            Self::Call { ref mut args, .. } |
            Self::CallIndirect { ref mut args, .. } |
            Self::MultiAry { ref mut args, .. } |
            Self::TryCall { ref mut args, .. } |
            Self::TryCallIndirect { ref mut args, .. } => {
                args.as_mut_slice(pool)
            }
        }
    }

    /// Compare two `InstructionData` for equality.
    ///
    /// This operation requires a reference to a `ValueListPool` to
    /// determine if the contents of any `ValueLists` are equal.
    ///
    /// This operation takes a closure that is allowed to map each
    /// argument value to some other value before the instructions
    /// are compared. This allows various forms of canonicalization.
    pub fn eq(&self, other: &Self, pool: &ir::ValueListPool) -> bool {
        if ::core::mem::discriminant(self) != ::core::mem::discriminant(other) {
            return false;
        }
        match (self, other) {
            (&Self::AtomicCas { opcode: ref opcode1, args: ref args1, flags: ref flags1 }, &Self::AtomicCas { opcode: ref opcode2, args: ref args2, flags: ref flags2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::AtomicRmw { opcode: ref opcode1, args: ref args1, flags: ref flags1, op: ref op1 }, &Self::AtomicRmw { opcode: ref opcode2, args: ref args2, flags: ref flags2, op: ref op2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && op1 == op2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Binary { opcode: ref opcode1, args: ref args1 }, &Self::Binary { opcode: ref opcode2, args: ref args2 }) =>  {
                opcode1 == opcode2
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::BinaryImm64 { opcode: ref opcode1, arg: ref arg1, imm: ref imm1 }, &Self::BinaryImm64 { opcode: ref opcode2, arg: ref arg2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::BinaryImm8 { opcode: ref opcode1, arg: ref arg1, imm: ref imm1 }, &Self::BinaryImm8 { opcode: ref opcode2, arg: ref arg2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::BranchTable { opcode: ref opcode1, arg: ref arg1, table: ref table1 }, &Self::BranchTable { opcode: ref opcode2, arg: ref arg2, table: ref table2 }) =>  {
                opcode1 == opcode2
                && table1 == table2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Brif { opcode: ref opcode1, arg: ref arg1, blocks: ref blocks1 }, &Self::Brif { opcode: ref opcode2, arg: ref arg2, blocks: ref blocks2 }) =>  {
                opcode1 == opcode2
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
                && blocks1.iter().zip(blocks2.iter()).all(|(a, b)| a.block(pool) == b.block(pool)) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:267
            }
            (&Self::Call { opcode: ref opcode1, args: ref args1, func_ref: ref func_ref1 }, &Self::Call { opcode: ref opcode2, args: ref args2, func_ref: ref func_ref2 }) =>  {
                opcode1 == opcode2
                && func_ref1 == func_ref2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.as_slice(pool).iter().zip(args2.as_slice(pool).iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::CallIndirect { opcode: ref opcode1, args: ref args1, sig_ref: ref sig_ref1 }, &Self::CallIndirect { opcode: ref opcode2, args: ref args2, sig_ref: ref sig_ref2 }) =>  {
                opcode1 == opcode2
                && sig_ref1 == sig_ref2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.as_slice(pool).iter().zip(args2.as_slice(pool).iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::CondTrap { opcode: ref opcode1, arg: ref arg1, code: ref code1 }, &Self::CondTrap { opcode: ref opcode2, arg: ref arg2, code: ref code2 }) =>  {
                opcode1 == opcode2
                && code1 == code2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::DynamicStackLoad { opcode: ref opcode1, dynamic_stack_slot: ref dynamic_stack_slot1 }, &Self::DynamicStackLoad { opcode: ref opcode2, dynamic_stack_slot: ref dynamic_stack_slot2 }) =>  {
                opcode1 == opcode2
                && dynamic_stack_slot1 == dynamic_stack_slot2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::DynamicStackStore { opcode: ref opcode1, arg: ref arg1, dynamic_stack_slot: ref dynamic_stack_slot1 }, &Self::DynamicStackStore { opcode: ref opcode2, arg: ref arg2, dynamic_stack_slot: ref dynamic_stack_slot2 }) =>  {
                opcode1 == opcode2
                && dynamic_stack_slot1 == dynamic_stack_slot2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::FloatCompare { opcode: ref opcode1, args: ref args1, cond: ref cond1 }, &Self::FloatCompare { opcode: ref opcode2, args: ref args2, cond: ref cond2 }) =>  {
                opcode1 == opcode2
                && cond1 == cond2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::FuncAddr { opcode: ref opcode1, func_ref: ref func_ref1 }, &Self::FuncAddr { opcode: ref opcode2, func_ref: ref func_ref2 }) =>  {
                opcode1 == opcode2
                && func_ref1 == func_ref2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::IntAddTrap { opcode: ref opcode1, args: ref args1, code: ref code1 }, &Self::IntAddTrap { opcode: ref opcode2, args: ref args2, code: ref code2 }) =>  {
                opcode1 == opcode2
                && code1 == code2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::IntCompare { opcode: ref opcode1, args: ref args1, cond: ref cond1 }, &Self::IntCompare { opcode: ref opcode2, args: ref args2, cond: ref cond2 }) =>  {
                opcode1 == opcode2
                && cond1 == cond2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::IntCompareImm { opcode: ref opcode1, arg: ref arg1, cond: ref cond1, imm: ref imm1 }, &Self::IntCompareImm { opcode: ref opcode2, arg: ref arg2, cond: ref cond2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && cond1 == cond2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Jump { opcode: ref opcode1, destination: ref destination1 }, &Self::Jump { opcode: ref opcode2, destination: ref destination2 }) =>  {
                opcode1 == opcode2
                && destination1 == destination2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:267
            }
            (&Self::Load { opcode: ref opcode1, arg: ref arg1, flags: ref flags1, offset: ref offset1 }, &Self::Load { opcode: ref opcode2, arg: ref arg2, flags: ref flags2, offset: ref offset2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && offset1 == offset2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::LoadNoOffset { opcode: ref opcode1, arg: ref arg1, flags: ref flags1 }, &Self::LoadNoOffset { opcode: ref opcode2, arg: ref arg2, flags: ref flags2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::MultiAry { opcode: ref opcode1, args: ref args1 }, &Self::MultiAry { opcode: ref opcode2, args: ref args2 }) =>  {
                opcode1 == opcode2
                && args1.as_slice(pool).iter().zip(args2.as_slice(pool).iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::NullAry { opcode: ref opcode1 }, &Self::NullAry { opcode: ref opcode2 }) =>  {
                opcode1 == opcode2
            }
            (&Self::Shuffle { opcode: ref opcode1, args: ref args1, imm: ref imm1 }, &Self::Shuffle { opcode: ref opcode2, args: ref args2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::StackLoad { opcode: ref opcode1, stack_slot: ref stack_slot1, offset: ref offset1 }, &Self::StackLoad { opcode: ref opcode2, stack_slot: ref stack_slot2, offset: ref offset2 }) =>  {
                opcode1 == opcode2
                && stack_slot1 == stack_slot2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && offset1 == offset2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::StackStore { opcode: ref opcode1, arg: ref arg1, stack_slot: ref stack_slot1, offset: ref offset1 }, &Self::StackStore { opcode: ref opcode2, arg: ref arg2, stack_slot: ref stack_slot2, offset: ref offset2 }) =>  {
                opcode1 == opcode2
                && stack_slot1 == stack_slot2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && offset1 == offset2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Store { opcode: ref opcode1, args: ref args1, flags: ref flags1, offset: ref offset1 }, &Self::Store { opcode: ref opcode2, args: ref args2, flags: ref flags2, offset: ref offset2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && offset1 == offset2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::StoreNoOffset { opcode: ref opcode1, args: ref args1, flags: ref flags1 }, &Self::StoreNoOffset { opcode: ref opcode2, args: ref args2, flags: ref flags2 }) =>  {
                opcode1 == opcode2
                && flags1 == flags2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Ternary { opcode: ref opcode1, args: ref args1 }, &Self::Ternary { opcode: ref opcode2, args: ref args2 }) =>  {
                opcode1 == opcode2
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::TernaryImm8 { opcode: ref opcode1, args: ref args1, imm: ref imm1 }, &Self::TernaryImm8 { opcode: ref opcode2, args: ref args2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.iter().zip(args2.iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Trap { opcode: ref opcode1, code: ref code1 }, &Self::Trap { opcode: ref opcode2, code: ref code2 }) =>  {
                opcode1 == opcode2
                && code1 == code2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::TryCall { opcode: ref opcode1, args: ref args1, func_ref: ref func_ref1, exception: ref exception1 }, &Self::TryCall { opcode: ref opcode2, args: ref args2, func_ref: ref func_ref2, exception: ref exception2 }) =>  {
                opcode1 == opcode2
                && func_ref1 == func_ref2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && exception1 == exception2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.as_slice(pool).iter().zip(args2.as_slice(pool).iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::TryCallIndirect { opcode: ref opcode1, args: ref args1, exception: ref exception1 }, &Self::TryCallIndirect { opcode: ref opcode2, args: ref args2, exception: ref exception2 }) =>  {
                opcode1 == opcode2
                && exception1 == exception2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
                && args1.as_slice(pool).iter().zip(args2.as_slice(pool).iter()).all(|(a, b)| a == b) // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::Unary { opcode: ref opcode1, arg: ref arg1 }, &Self::Unary { opcode: ref opcode2, arg: ref arg2 }) =>  {
                opcode1 == opcode2
                && arg1 == arg2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:264
            }
            (&Self::UnaryConst { opcode: ref opcode1, constant_handle: ref constant_handle1 }, &Self::UnaryConst { opcode: ref opcode2, constant_handle: ref constant_handle2 }) =>  {
                opcode1 == opcode2
                && constant_handle1 == constant_handle2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::UnaryGlobalValue { opcode: ref opcode1, global_value: ref global_value1 }, &Self::UnaryGlobalValue { opcode: ref opcode2, global_value: ref global_value2 }) =>  {
                opcode1 == opcode2
                && global_value1 == global_value2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::UnaryIeee16 { opcode: ref opcode1, imm: ref imm1 }, &Self::UnaryIeee16 { opcode: ref opcode2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::UnaryIeee32 { opcode: ref opcode1, imm: ref imm1 }, &Self::UnaryIeee32 { opcode: ref opcode2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::UnaryIeee64 { opcode: ref opcode1, imm: ref imm1 }, &Self::UnaryIeee64 { opcode: ref opcode2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            (&Self::UnaryImm { opcode: ref opcode1, imm: ref imm1 }, &Self::UnaryImm { opcode: ref opcode2, imm: ref imm2 }) =>  {
                opcode1 == opcode2
                && imm1 == imm2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:261
            }
            _ => unreachable!()
        }
    }

    /// Hash an `InstructionData`.
    ///
    /// This operation requires a reference to a `ValueListPool` to
    /// hash the contents of any `ValueLists`.
    ///
    /// This operation takes a closure that is allowed to map each
    /// argument value to some other value before it is hashed. This
    /// allows various forms of canonicalization.
    pub fn hash<H: ::core::hash::Hasher>(&self, state: &mut H, pool: &ir::ValueListPool) {
        match *self {
            Self::AtomicCas{opcode, ref args, flags} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::AtomicRmw{opcode, ref args, flags, op} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&op, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Binary{opcode, ref args} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::BinaryImm64{opcode, ref arg, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::BinaryImm8{opcode, ref arg, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::BranchTable{opcode, ref arg, table} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&table, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Brif{opcode, ref arg, ref blocks} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
                ::core::hash::Hash::hash(&blocks.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:336
                for &block in blocks {
                    ::core::hash::Hash::hash(&block.block(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:338
                    for arg in block.args(pool) {
                        ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:340
                    }
                }
            }
            Self::Call{opcode, ref args, func_ref} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&func_ref, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args.as_slice(pool) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::CallIndirect{opcode, ref args, sig_ref} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&sig_ref, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args.as_slice(pool) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::CondTrap{opcode, ref arg, code} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&code, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::DynamicStackLoad{opcode, dynamic_stack_slot} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&dynamic_stack_slot, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::DynamicStackStore{opcode, ref arg, dynamic_stack_slot} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&dynamic_stack_slot, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::FloatCompare{opcode, ref args, cond} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&cond, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::FuncAddr{opcode, func_ref} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&func_ref, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::IntAddTrap{opcode, ref args, code} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&code, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::IntCompare{opcode, ref args, cond} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&cond, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::IntCompareImm{opcode, ref arg, cond, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&cond, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Jump{opcode, ref destination} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:336
                for &block in std::slice::from_ref(destination) {
                    ::core::hash::Hash::hash(&block.block(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:338
                    for arg in block.args(pool) {
                        ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:340
                    }
                }
            }
            Self::Load{opcode, ref arg, flags, offset} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&offset, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::LoadNoOffset{opcode, ref arg, flags} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::MultiAry{opcode, ref args} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&args.len(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args.as_slice(pool) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::NullAry{opcode} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::Shuffle{opcode, ref args, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::StackLoad{opcode, stack_slot, offset} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&stack_slot, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&offset, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::StackStore{opcode, ref arg, stack_slot, offset} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&stack_slot, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&offset, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Store{opcode, ref args, flags, offset} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&offset, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::StoreNoOffset{opcode, ref args, flags} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&flags, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Ternary{opcode, ref args} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::TernaryImm8{opcode, ref args, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Trap{opcode, code} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&code, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::TryCall{opcode, ref args, func_ref, exception} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&func_ref, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&exception, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args.as_slice(pool) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::TryCallIndirect{opcode, ref args, exception} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&exception, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&args.len(pool), state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in args.as_slice(pool) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::Unary{opcode, ref arg} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&1, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
                for &arg in std::slice::from_ref(arg) {
                    ::core::hash::Hash::hash(&arg, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:331
                }
            }
            Self::UnaryConst{opcode, constant_handle} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&constant_handle, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::UnaryGlobalValue{opcode, global_value} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&global_value, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::UnaryIeee16{opcode, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::UnaryIeee32{opcode, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::UnaryIeee64{opcode, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
            Self::UnaryImm{opcode, imm} =>  {
                ::core::hash::Hash::hash( &::core::mem::discriminant(self), state);
                ::core::hash::Hash::hash(&opcode, state);
                ::core::hash::Hash::hash(&imm, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:326
                ::core::hash::Hash::hash(&0, state); // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:328
            }
        }
    }

    /// Deep-clone an `InstructionData`, including any referenced lists.
    ///
    /// This operation requires a reference to a `ValueListPool` to
    /// clone the `ValueLists`.
    pub fn deep_clone(&self, pool: &mut ir::ValueListPool) -> Self {
        match *self {
            Self::AtomicCas{opcode, args, flags} =>  {
                Self::AtomicCas {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::AtomicRmw{opcode, args, flags, op} =>  {
                Self::AtomicRmw {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    op, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Binary{opcode, args} =>  {
                Self::Binary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                }
            }
            Self::BinaryImm64{opcode, arg, imm} =>  {
                Self::BinaryImm64 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::BinaryImm8{opcode, arg, imm} =>  {
                Self::BinaryImm8 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::BranchTable{opcode, arg, table} =>  {
                Self::BranchTable {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    table, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Brif{opcode, arg, blocks} =>  {
                Self::Brif {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    blocks: [blocks[0].deep_clone(pool), blocks[1].deep_clone(pool)], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:404
                }
            }
            Self::Call{opcode, ref args, func_ref} =>  {
                Self::Call {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args: args.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:391
                    func_ref, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::CallIndirect{opcode, ref args, sig_ref} =>  {
                Self::CallIndirect {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args: args.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:391
                    sig_ref, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::CondTrap{opcode, arg, code} =>  {
                Self::CondTrap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::DynamicStackLoad{opcode, dynamic_stack_slot} =>  {
                Self::DynamicStackLoad {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    dynamic_stack_slot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::DynamicStackStore{opcode, arg, dynamic_stack_slot} =>  {
                Self::DynamicStackStore {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    dynamic_stack_slot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::FloatCompare{opcode, args, cond} =>  {
                Self::FloatCompare {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::FuncAddr{opcode, func_ref} =>  {
                Self::FuncAddr {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    func_ref, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::IntAddTrap{opcode, args, code} =>  {
                Self::IntAddTrap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::IntCompare{opcode, args, cond} =>  {
                Self::IntCompare {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::IntCompareImm{opcode, arg, cond, imm} =>  {
                Self::IntCompareImm {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Jump{opcode, destination} =>  {
                Self::Jump {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    destination: destination.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:401
                }
            }
            Self::Load{opcode, arg, flags, offset} =>  {
                Self::Load {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::LoadNoOffset{opcode, arg, flags} =>  {
                Self::LoadNoOffset {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::MultiAry{opcode, ref args} =>  {
                Self::MultiAry {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args: args.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:391
                }
            }
            Self::NullAry{opcode} =>  {
                Self::NullAry {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                }
            }
            Self::Shuffle{opcode, args, imm} =>  {
                Self::Shuffle {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::StackLoad{opcode, stack_slot, offset} =>  {
                Self::StackLoad {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    stack_slot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::StackStore{opcode, arg, stack_slot, offset} =>  {
                Self::StackStore {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                    stack_slot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Store{opcode, args, flags, offset} =>  {
                Self::Store {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::StoreNoOffset{opcode, args, flags} =>  {
                Self::StoreNoOffset {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Ternary{opcode, args} =>  {
                Self::Ternary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                }
            }
            Self::TernaryImm8{opcode, args, imm} =>  {
                Self::TernaryImm8 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:395
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Trap{opcode, code} =>  {
                Self::Trap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::TryCall{opcode, ref args, func_ref, exception} =>  {
                Self::TryCall {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args: args.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:391
                    func_ref, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                    exception, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::TryCallIndirect{opcode, ref args, exception} =>  {
                Self::TryCallIndirect {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    args: args.deep_clone(pool), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:391
                    exception, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::Unary{opcode, arg} =>  {
                Self::Unary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    arg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:393
                }
            }
            Self::UnaryConst{opcode, constant_handle} =>  {
                Self::UnaryConst {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    constant_handle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::UnaryGlobalValue{opcode, global_value} =>  {
                Self::UnaryGlobalValue {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    global_value, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::UnaryIeee16{opcode, imm} =>  {
                Self::UnaryIeee16 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::UnaryIeee32{opcode, imm} =>  {
                Self::UnaryIeee32 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::UnaryIeee64{opcode, imm} =>  {
                Self::UnaryIeee64 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
            Self::UnaryImm{opcode, imm} =>  {
                Self::UnaryImm {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:388
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:410
                }
            }
        }
    }
    /// Map some functions, described by the given `InstructionMapper`, over each of the
    /// entities within this instruction, producing a new `InstructionData`.
    pub fn map(&self, mut mapper: impl crate::ir::instructions::InstructionMapper) -> Self {
        match *self {
            Self::AtomicCas{opcode, args, flags} =>  {
                Self::AtomicCas {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1]), mapper.map_value(args[2])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::AtomicRmw{opcode, args, flags, op} =>  {
                Self::AtomicRmw {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                    op, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::Binary{opcode, args} =>  {
                Self::Binary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                }
            }
            Self::BinaryImm64{opcode, arg, imm} =>  {
                Self::BinaryImm64 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::BinaryImm8{opcode, arg, imm} =>  {
                Self::BinaryImm8 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::BranchTable{opcode, arg, table} =>  {
                Self::BranchTable {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    table: mapper.map_jump_table(table), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::Brif{opcode, arg, blocks} =>  {
                Self::Brif {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    blocks: [mapper.map_block_call(blocks[0]), mapper.map_block_call(blocks[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:472
                }
            }
            Self::Call{opcode, args, func_ref} =>  {
                Self::Call {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: mapper.map_value_list(args), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:455
                    func_ref: mapper.map_func_ref(func_ref), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::CallIndirect{opcode, args, sig_ref} =>  {
                Self::CallIndirect {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: mapper.map_value_list(args), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:455
                    sig_ref: mapper.map_sig_ref(sig_ref), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::CondTrap{opcode, arg, code} =>  {
                Self::CondTrap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::DynamicStackLoad{opcode, dynamic_stack_slot} =>  {
                Self::DynamicStackLoad {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    dynamic_stack_slot: mapper.map_dynamic_stack_slot(dynamic_stack_slot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::DynamicStackStore{opcode, arg, dynamic_stack_slot} =>  {
                Self::DynamicStackStore {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    dynamic_stack_slot: mapper.map_dynamic_stack_slot(dynamic_stack_slot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::FloatCompare{opcode, args, cond} =>  {
                Self::FloatCompare {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::FuncAddr{opcode, func_ref} =>  {
                Self::FuncAddr {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    func_ref: mapper.map_func_ref(func_ref), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::IntAddTrap{opcode, args, code} =>  {
                Self::IntAddTrap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::IntCompare{opcode, args, cond} =>  {
                Self::IntCompare {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::IntCompareImm{opcode, arg, cond, imm} =>  {
                Self::IntCompareImm {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    cond, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::Jump{opcode, destination} =>  {
                Self::Jump {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    destination: mapper.map_block_call(destination), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:469
                }
            }
            Self::Load{opcode, arg, flags, offset} =>  {
                Self::Load {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::LoadNoOffset{opcode, arg, flags} =>  {
                Self::LoadNoOffset {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::MultiAry{opcode, args} =>  {
                Self::MultiAry {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: mapper.map_value_list(args), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:455
                }
            }
            Self::NullAry{opcode} =>  {
                Self::NullAry {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                }
            }
            Self::Shuffle{opcode, args, imm} =>  {
                Self::Shuffle {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    imm: mapper.map_immediate(imm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::StackLoad{opcode, stack_slot, offset} =>  {
                Self::StackLoad {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    stack_slot: mapper.map_stack_slot(stack_slot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::StackStore{opcode, arg, stack_slot, offset} =>  {
                Self::StackStore {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                    stack_slot: mapper.map_stack_slot(stack_slot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::Store{opcode, args, flags, offset} =>  {
                Self::Store {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                    offset, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::StoreNoOffset{opcode, args, flags} =>  {
                Self::StoreNoOffset {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    flags, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::Ternary{opcode, args} =>  {
                Self::Ternary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1]), mapper.map_value(args[2])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                }
            }
            Self::TernaryImm8{opcode, args, imm} =>  {
                Self::TernaryImm8 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: [mapper.map_value(args[0]), mapper.map_value(args[1])], // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:463
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::Trap{opcode, code} =>  {
                Self::Trap {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    code, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::TryCall{opcode, args, func_ref, exception} =>  {
                Self::TryCall {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: mapper.map_value_list(args), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:455
                    func_ref: mapper.map_func_ref(func_ref), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                    exception: mapper.map_exception_table(exception), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::TryCallIndirect{opcode, args, exception} =>  {
                Self::TryCallIndirect {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    args: mapper.map_value_list(args), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:455
                    exception: mapper.map_exception_table(exception), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::Unary{opcode, arg} =>  {
                Self::Unary {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    arg: mapper.map_value(arg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:457
                }
            }
            Self::UnaryConst{opcode, constant_handle} =>  {
                Self::UnaryConst {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    constant_handle: mapper.map_constant(constant_handle), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::UnaryGlobalValue{opcode, global_value} =>  {
                Self::UnaryGlobalValue {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    global_value: mapper.map_global_value(global_value), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:492
                }
            }
            Self::UnaryIeee16{opcode, imm} =>  {
                Self::UnaryIeee16 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::UnaryIeee32{opcode, imm} =>  {
                Self::UnaryIeee32 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::UnaryIeee64{opcode, imm} =>  {
                Self::UnaryIeee64 {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
            Self::UnaryImm{opcode, imm} =>  {
                Self::UnaryImm {
                    opcode, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:452
                    imm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:499
                }
            }
        }
    }
}

/// An instruction opcode.
///
/// All instructions from all supported ISAs are present.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
            feature = "enable-serde",
            derive(serde_derive::Serialize, serde_derive::Deserialize)
        )]
pub enum Opcode {
    /// `jump block_call`. (Jump)
    Jump = 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:570
    /// `brif c, block_then, block_else`. (Brif)
    /// Type inferred from `c`.
    Brif, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `br_table x, JT`. (BranchTable)
    BrTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `debugtrap`. (NullAry)
    Debugtrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `trap code`. (Trap)
    Trap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `trapz c, code`. (CondTrap)
    /// Type inferred from `c`.
    Trapz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `trapnz c, code`. (CondTrap)
    /// Type inferred from `c`.
    Trapnz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `return rvals`. (MultiAry)
    Return, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `rvals = call FN, args`. (Call)
    Call, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `rvals = call_indirect SIG, callee, args`. (CallIndirect)
    /// Type inferred from `callee`.
    CallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `return_call FN, args`. (Call)
    ReturnCall, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `return_call_indirect SIG, callee, args`. (CallIndirect)
    /// Type inferred from `callee`.
    ReturnCallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = func_addr FN`. (FuncAddr)
    FuncAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `try_call callee, args, ET`. (TryCall)
    TryCall, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `try_call_indirect callee, args, ET`. (TryCallIndirect)
    /// Type inferred from `callee`.
    TryCallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = splat x`. (Unary)
    Splat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = swizzle x, y`. (Binary)
    Swizzle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = x86_pshufb x, y`. (Binary)
    X86Pshufb, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = insertlane x, y, Idx`. (TernaryImm8)
    /// Type inferred from `x`.
    Insertlane, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = extractlane x, Idx`. (BinaryImm8)
    /// Type inferred from `x`.
    Extractlane, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = smin x, y`. (Binary)
    /// Type inferred from `x`.
    Smin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = umin x, y`. (Binary)
    /// Type inferred from `x`.
    Umin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = smax x, y`. (Binary)
    /// Type inferred from `x`.
    Smax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = umax x, y`. (Binary)
    /// Type inferred from `x`.
    Umax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = avg_round x, y`. (Binary)
    /// Type inferred from `x`.
    AvgRound, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uadd_sat x, y`. (Binary)
    /// Type inferred from `x`.
    UaddSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sadd_sat x, y`. (Binary)
    /// Type inferred from `x`.
    SaddSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = usub_sat x, y`. (Binary)
    /// Type inferred from `x`.
    UsubSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ssub_sat x, y`. (Binary)
    /// Type inferred from `x`.
    SsubSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = load MemFlags, p, Offset`. (Load)
    Load, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `store MemFlags, x, p, Offset`. (Store)
    /// Type inferred from `x`.
    Store, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload8 MemFlags, p, Offset`. (Load)
    Uload8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload8 MemFlags, p, Offset`. (Load)
    Sload8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `istore8 MemFlags, x, p, Offset`. (Store)
    /// Type inferred from `x`.
    Istore8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload16 MemFlags, p, Offset`. (Load)
    Uload16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload16 MemFlags, p, Offset`. (Load)
    Sload16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `istore16 MemFlags, x, p, Offset`. (Store)
    /// Type inferred from `x`.
    Istore16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload32 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Uload32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload32 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Sload32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `istore32 MemFlags, x, p, Offset`. (Store)
    /// Type inferred from `x`.
    Istore32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `out_payload0 = stack_switch store_context_ptr, load_context_ptr, in_payload0`. (Ternary)
    /// Type inferred from `load_context_ptr`.
    StackSwitch, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload8x8 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Uload8x8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload8x8 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Sload8x8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload16x4 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Uload16x4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload16x4 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Sload16x4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uload32x2 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Uload32x2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sload32x2 MemFlags, p, Offset`. (Load)
    /// Type inferred from `p`.
    Sload32x2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = stack_load SS, Offset`. (StackLoad)
    StackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `stack_store x, SS, Offset`. (StackStore)
    /// Type inferred from `x`.
    StackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = stack_addr SS, Offset`. (StackLoad)
    StackAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = dynamic_stack_load DSS`. (DynamicStackLoad)
    DynamicStackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `dynamic_stack_store x, DSS`. (DynamicStackStore)
    /// Type inferred from `x`.
    DynamicStackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = dynamic_stack_addr DSS`. (DynamicStackLoad)
    DynamicStackAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = global_value GV`. (UnaryGlobalValue)
    GlobalValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = symbol_value GV`. (UnaryGlobalValue)
    SymbolValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = tls_value GV`. (UnaryGlobalValue)
    TlsValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = get_pinned_reg`. (NullAry)
    GetPinnedReg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `set_pinned_reg addr`. (Unary)
    /// Type inferred from `addr`.
    SetPinnedReg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = get_frame_pointer`. (NullAry)
    GetFramePointer, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = get_stack_pointer`. (NullAry)
    GetStackPointer, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `addr = get_return_address`. (NullAry)
    GetReturnAddress, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iconst N`. (UnaryImm)
    Iconst, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = f16const N`. (UnaryIeee16)
    F16const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = f32const N`. (UnaryIeee32)
    F32const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = f64const N`. (UnaryIeee64)
    F64const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = f128const N`. (UnaryConst)
    F128const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = vconst N`. (UnaryConst)
    Vconst, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = shuffle a, b, mask`. (Shuffle)
    Shuffle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `nop`. (NullAry)
    Nop, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = select c, x, y`. (Ternary)
    /// Type inferred from `x`.
    Select, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = select_spectre_guard c, x, y`. (Ternary)
    /// Type inferred from `x`.
    SelectSpectreGuard, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bitselect c, x, y`. (Ternary)
    /// Type inferred from `x`.
    Bitselect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = x86_blendv c, x, y`. (Ternary)
    /// Type inferred from `x`.
    X86Blendv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `s = vany_true a`. (Unary)
    /// Type inferred from `a`.
    VanyTrue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `s = vall_true a`. (Unary)
    /// Type inferred from `a`.
    VallTrue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `x = vhigh_bits a`. (Unary)
    VhighBits, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = icmp Cond, x, y`. (IntCompare)
    /// Type inferred from `x`.
    Icmp, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = icmp_imm Cond, x, Y`. (IntCompareImm)
    /// Type inferred from `x`.
    IcmpImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iadd x, y`. (Binary)
    /// Type inferred from `x`.
    Iadd, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = isub x, y`. (Binary)
    /// Type inferred from `x`.
    Isub, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ineg x`. (Unary)
    /// Type inferred from `x`.
    Ineg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iabs x`. (Unary)
    /// Type inferred from `x`.
    Iabs, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = imul x, y`. (Binary)
    /// Type inferred from `x`.
    Imul, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = umulhi x, y`. (Binary)
    /// Type inferred from `x`.
    Umulhi, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = smulhi x, y`. (Binary)
    /// Type inferred from `x`.
    Smulhi, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sqmul_round_sat x, y`. (Binary)
    /// Type inferred from `x`.
    SqmulRoundSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = x86_pmulhrsw x, y`. (Binary)
    /// Type inferred from `x`.
    X86Pmulhrsw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = udiv x, y`. (Binary)
    /// Type inferred from `x`.
    Udiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sdiv x, y`. (Binary)
    /// Type inferred from `x`.
    Sdiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = urem x, y`. (Binary)
    /// Type inferred from `x`.
    Urem, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = srem x, y`. (Binary)
    /// Type inferred from `x`.
    Srem, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iadd_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    IaddImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = imul_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    ImulImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = udiv_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    UdivImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sdiv_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    SdivImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = urem_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    UremImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = srem_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    SremImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = irsub_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    IrsubImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, c_out = sadd_overflow_cin x, y, c_in`. (Ternary)
    /// Type inferred from `y`.
    SaddOverflowCin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, c_out = uadd_overflow_cin x, y, c_in`. (Ternary)
    /// Type inferred from `y`.
    UaddOverflowCin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = uadd_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    UaddOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = sadd_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    SaddOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = usub_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    UsubOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = ssub_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    SsubOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = umul_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    UmulOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, of = smul_overflow x, y`. (Binary)
    /// Type inferred from `x`.
    SmulOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uadd_overflow_trap x, y, code`. (IntAddTrap)
    /// Type inferred from `x`.
    UaddOverflowTrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, b_out = ssub_overflow_bin x, y, b_in`. (Ternary)
    /// Type inferred from `y`.
    SsubOverflowBin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a, b_out = usub_overflow_bin x, y, b_in`. (Ternary)
    /// Type inferred from `y`.
    UsubOverflowBin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = band x, y`. (Binary)
    /// Type inferred from `x`.
    Band, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bor x, y`. (Binary)
    /// Type inferred from `x`.
    Bor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bxor x, y`. (Binary)
    /// Type inferred from `x`.
    Bxor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bnot x`. (Unary)
    /// Type inferred from `x`.
    Bnot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = band_not x, y`. (Binary)
    /// Type inferred from `x`.
    BandNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bor_not x, y`. (Binary)
    /// Type inferred from `x`.
    BorNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bxor_not x, y`. (Binary)
    /// Type inferred from `x`.
    BxorNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = band_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    BandImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bor_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    BorImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bxor_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    BxorImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = rotl x, y`. (Binary)
    /// Type inferred from `x`.
    Rotl, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = rotr x, y`. (Binary)
    /// Type inferred from `x`.
    Rotr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = rotl_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    RotlImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = rotr_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    RotrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ishl x, y`. (Binary)
    /// Type inferred from `x`.
    Ishl, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ushr x, y`. (Binary)
    /// Type inferred from `x`.
    Ushr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sshr x, y`. (Binary)
    /// Type inferred from `x`.
    Sshr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ishl_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    IshlImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ushr_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    UshrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sshr_imm x, Y`. (BinaryImm64)
    /// Type inferred from `x`.
    SshrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bitrev x`. (Unary)
    /// Type inferred from `x`.
    Bitrev, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = clz x`. (Unary)
    /// Type inferred from `x`.
    Clz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = cls x`. (Unary)
    /// Type inferred from `x`.
    Cls, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ctz x`. (Unary)
    /// Type inferred from `x`.
    Ctz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bswap x`. (Unary)
    /// Type inferred from `x`.
    Bswap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = popcnt x`. (Unary)
    /// Type inferred from `x`.
    Popcnt, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcmp Cond, x, y`. (FloatCompare)
    /// Type inferred from `x`.
    Fcmp, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fadd x, y`. (Binary)
    /// Type inferred from `x`.
    Fadd, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fsub x, y`. (Binary)
    /// Type inferred from `x`.
    Fsub, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fmul x, y`. (Binary)
    /// Type inferred from `x`.
    Fmul, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fdiv x, y`. (Binary)
    /// Type inferred from `x`.
    Fdiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sqrt x`. (Unary)
    /// Type inferred from `x`.
    Sqrt, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fma x, y, z`. (Ternary)
    /// Type inferred from `y`.
    Fma, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fneg x`. (Unary)
    /// Type inferred from `x`.
    Fneg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fabs x`. (Unary)
    /// Type inferred from `x`.
    Fabs, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcopysign x, y`. (Binary)
    /// Type inferred from `x`.
    Fcopysign, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fmin x, y`. (Binary)
    /// Type inferred from `x`.
    Fmin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fmax x, y`. (Binary)
    /// Type inferred from `x`.
    Fmax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ceil x`. (Unary)
    /// Type inferred from `x`.
    Ceil, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = floor x`. (Unary)
    /// Type inferred from `x`.
    Floor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = trunc x`. (Unary)
    /// Type inferred from `x`.
    Trunc, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = nearest x`. (Unary)
    /// Type inferred from `x`.
    Nearest, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bitcast MemFlags, x`. (LoadNoOffset)
    Bitcast, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = scalar_to_vector s`. (Unary)
    ScalarToVector, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = bmask x`. (Unary)
    Bmask, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = ireduce x`. (Unary)
    Ireduce, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = snarrow x, y`. (Binary)
    /// Type inferred from `x`.
    Snarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = unarrow x, y`. (Binary)
    /// Type inferred from `x`.
    Unarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uunarrow x, y`. (Binary)
    /// Type inferred from `x`.
    Uunarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = swiden_low x`. (Unary)
    /// Type inferred from `x`.
    SwidenLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = swiden_high x`. (Unary)
    /// Type inferred from `x`.
    SwidenHigh, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uwiden_low x`. (Unary)
    /// Type inferred from `x`.
    UwidenLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uwiden_high x`. (Unary)
    /// Type inferred from `x`.
    UwidenHigh, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iadd_pairwise x, y`. (Binary)
    /// Type inferred from `x`.
    IaddPairwise, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = x86_pmaddubsw x, y`. (Binary)
    X86Pmaddubsw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = uextend x`. (Unary)
    Uextend, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = sextend x`. (Unary)
    Sextend, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fpromote x`. (Unary)
    Fpromote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fdemote x`. (Unary)
    Fdemote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fvdemote x`. (Unary)
    Fvdemote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `x = fvpromote_low a`. (Unary)
    FvpromoteLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_to_uint x`. (Unary)
    FcvtToUint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_to_sint x`. (Unary)
    FcvtToSint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_to_uint_sat x`. (Unary)
    FcvtToUintSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_to_sint_sat x`. (Unary)
    FcvtToSintSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = x86_cvtt2dq x`. (Unary)
    X86Cvtt2dq, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_from_uint x`. (Unary)
    FcvtFromUint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = fcvt_from_sint x`. (Unary)
    FcvtFromSint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `lo, hi = isplit x`. (Unary)
    /// Type inferred from `x`.
    Isplit, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = iconcat lo, hi`. (Binary)
    /// Type inferred from `lo`.
    Iconcat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = atomic_rmw MemFlags, AtomicRmwOp, p, x`. (AtomicRmw)
    AtomicRmw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = atomic_cas MemFlags, p, e, x`. (AtomicCas)
    /// Type inferred from `x`.
    AtomicCas, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = atomic_load MemFlags, p`. (LoadNoOffset)
    AtomicLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `atomic_store MemFlags, x, p`. (StoreNoOffset)
    /// Type inferred from `x`.
    AtomicStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `fence`. (NullAry)
    Fence, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
    /// `a = extract_vector x, y`. (BinaryImm8)
    /// Type inferred from `x`.
    ExtractVector, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:573
}

impl Opcode {
    /// True for instructions that terminate the block
    pub fn is_terminator(self) -> bool {
        match self {
            Self::BrTable |
            Self::Brif |
            Self::Jump |
            Self::Return |
            Self::ReturnCall |
            Self::ReturnCallIndirect |
            Self::Trap |
            Self::TryCall |
            Self::TryCallIndirect => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// True for all branch or jump instructions.
    pub fn is_branch(self) -> bool {
        match self {
            Self::BrTable |
            Self::Brif |
            Self::Jump |
            Self::TryCall |
            Self::TryCallIndirect => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Is this a call instruction?
    pub fn is_call(self) -> bool {
        match self {
            Self::Call |
            Self::CallIndirect |
            Self::ReturnCall |
            Self::ReturnCallIndirect |
            Self::StackSwitch |
            Self::TryCall |
            Self::TryCallIndirect => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Is this a return instruction?
    pub fn is_return(self) -> bool {
        match self {
            Self::Return |
            Self::ReturnCall |
            Self::ReturnCallIndirect => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Can this instruction read from memory?
    pub fn can_load(self) -> bool {
        match self {
            Self::AtomicCas |
            Self::AtomicLoad |
            Self::AtomicRmw |
            Self::Debugtrap |
            Self::DynamicStackLoad |
            Self::Load |
            Self::Sload16 |
            Self::Sload16x4 |
            Self::Sload32 |
            Self::Sload32x2 |
            Self::Sload8 |
            Self::Sload8x8 |
            Self::StackLoad |
            Self::StackSwitch |
            Self::Uload16 |
            Self::Uload16x4 |
            Self::Uload32 |
            Self::Uload32x2 |
            Self::Uload8 |
            Self::Uload8x8 => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Can this instruction write to memory?
    pub fn can_store(self) -> bool {
        match self {
            Self::AtomicCas |
            Self::AtomicRmw |
            Self::AtomicStore |
            Self::Debugtrap |
            Self::DynamicStackStore |
            Self::Istore16 |
            Self::Istore32 |
            Self::Istore8 |
            Self::StackStore |
            Self::StackSwitch |
            Self::Store => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Can this instruction cause a trap?
    pub fn can_trap(self) -> bool {
        match self {
            Self::FcvtToSint |
            Self::FcvtToUint |
            Self::Sdiv |
            Self::Srem |
            Self::Trap |
            Self::Trapnz |
            Self::Trapz |
            Self::UaddOverflowTrap |
            Self::Udiv |
            Self::Urem => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Does this instruction have other side effects besides can_* flags?
    pub fn other_side_effects(self) -> bool {
        match self {
            Self::AtomicCas |
            Self::AtomicLoad |
            Self::AtomicRmw |
            Self::AtomicStore |
            Self::Debugtrap |
            Self::Fence |
            Self::GetPinnedReg |
            Self::SetPinnedReg |
            Self::StackSwitch => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// Despite having side effects, is this instruction okay to GVN?
    pub fn side_effects_idempotent(self) -> bool {
        match self {
            Self::FcvtToSint |
            Self::FcvtToUint |
            Self::Sdiv |
            Self::Srem |
            Self::Trapnz |
            Self::Trapz |
            Self::UaddOverflowTrap |
            Self::Udiv |
            Self::Urem => {
                true
            }
            _ => {
                false
            }
        }
    }

    /// All cranelift opcodes.
    pub fn all() -> &'static [Opcode] {
        return &[
            Opcode::Jump, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Brif, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BrTable, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Debugtrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Trap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Trapz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Trapnz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Return, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Call, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::CallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::ReturnCall, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::ReturnCallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FuncAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::TryCall, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::TryCallIndirect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Splat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Swizzle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::X86Pshufb, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Insertlane, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Extractlane, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Smin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Umin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Smax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Umax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::AvgRound, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UaddSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SaddSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UsubSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SsubSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Load, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Store, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Istore8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Istore16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Istore32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::StackSwitch, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload8x8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload8x8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload16x4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload16x4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uload32x2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sload32x2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::StackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::StackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::StackAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::DynamicStackLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::DynamicStackStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::DynamicStackAddr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::GlobalValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SymbolValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::TlsValue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::GetPinnedReg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SetPinnedReg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::GetFramePointer, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::GetStackPointer, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::GetReturnAddress, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Iconst, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::F16const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::F32const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::F64const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::F128const, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Vconst, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Shuffle, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Nop, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Select, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SelectSpectreGuard, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bitselect, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::X86Blendv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::VanyTrue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::VallTrue, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::VhighBits, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Icmp, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::IcmpImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Iadd, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Isub, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ineg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Iabs, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Imul, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Umulhi, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Smulhi, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SqmulRoundSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::X86Pmulhrsw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Udiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sdiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Urem, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Srem, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::IaddImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::ImulImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UdivImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SdivImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UremImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SremImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::IrsubImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SaddOverflowCin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UaddOverflowCin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UaddOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SaddOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UsubOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SsubOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UmulOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SmulOverflow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UaddOverflowTrap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SsubOverflowBin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UsubOverflowBin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Band, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bxor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bnot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BandNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BorNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BxorNot, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BandImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BorImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::BxorImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Rotl, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Rotr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::RotlImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::RotrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ishl, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ushr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sshr, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::IshlImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UshrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SshrImm, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bitrev, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Clz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Cls, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ctz, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bswap, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Popcnt, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fcmp, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fadd, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fsub, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fmul, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fdiv, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sqrt, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fma, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fneg, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fabs, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fcopysign, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fmin, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fmax, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ceil, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Floor, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Trunc, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Nearest, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bitcast, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::ScalarToVector, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Bmask, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Ireduce, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Snarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Unarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uunarrow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SwidenLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::SwidenHigh, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UwidenLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::UwidenHigh, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::IaddPairwise, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::X86Pmaddubsw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Uextend, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Sextend, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fpromote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fdemote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fvdemote, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FvpromoteLow, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtToUint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtToSint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtToUintSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtToSintSat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::X86Cvtt2dq, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtFromUint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::FcvtFromSint, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Isplit, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Iconcat, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::AtomicRmw, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::AtomicCas, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::AtomicLoad, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::AtomicStore, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::Fence, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
            Opcode::ExtractVector, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:650
        ];
    }

}

const OPCODE_FORMAT: [InstructionFormat; 185] = [ // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:660
    InstructionFormat::Jump, // jump // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Brif, // brif // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BranchTable, // br_table // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // debugtrap // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Trap, // trap // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::CondTrap, // trapz // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::CondTrap, // trapnz // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::MultiAry, // return // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Call, // call // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::CallIndirect, // call_indirect // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Call, // return_call // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::CallIndirect, // return_call_indirect // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::FuncAddr, // func_addr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::TryCall, // try_call // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::TryCallIndirect, // try_call_indirect // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // splat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // swizzle // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // x86_pshufb // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::TernaryImm8, // insertlane // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm8, // extractlane // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // smin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // umin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // smax // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // umax // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // avg_round // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // uadd_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // sadd_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // usub_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // ssub_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // load // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Store, // store // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload8 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload8 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Store, // istore8 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload16 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload16 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Store, // istore16 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload32 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload32 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Store, // istore32 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // stack_switch // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload8x8 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload8x8 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload16x4 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload16x4 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // uload32x2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Load, // sload32x2 // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::StackLoad, // stack_load // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::StackStore, // stack_store // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::StackLoad, // stack_addr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::DynamicStackLoad, // dynamic_stack_load // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::DynamicStackStore, // dynamic_stack_store // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::DynamicStackLoad, // dynamic_stack_addr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryGlobalValue, // global_value // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryGlobalValue, // symbol_value // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryGlobalValue, // tls_value // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // get_pinned_reg // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // set_pinned_reg // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // get_frame_pointer // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // get_stack_pointer // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // get_return_address // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryImm, // iconst // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryIeee16, // f16const // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryIeee32, // f32const // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryIeee64, // f64const // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryConst, // f128const // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::UnaryConst, // vconst // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Shuffle, // shuffle // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // nop // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // select // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // select_spectre_guard // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // bitselect // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // x86_blendv // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // vany_true // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // vall_true // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // vhigh_bits // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::IntCompare, // icmp // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::IntCompareImm, // icmp_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // iadd // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // isub // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // ineg // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // iabs // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // imul // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // umulhi // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // smulhi // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // sqmul_round_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // x86_pmulhrsw // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // udiv // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // sdiv // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // urem // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // srem // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // iadd_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // imul_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // udiv_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // sdiv_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // urem_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // srem_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // irsub_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // sadd_overflow_cin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // uadd_overflow_cin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // uadd_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // sadd_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // usub_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // ssub_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // umul_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // smul_overflow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::IntAddTrap, // uadd_overflow_trap // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // ssub_overflow_bin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // usub_overflow_bin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // band // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // bor // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // bxor // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // bnot // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // band_not // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // bor_not // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // bxor_not // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // band_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // bor_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // bxor_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // rotl // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // rotr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // rotl_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // rotr_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // ishl // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // ushr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // sshr // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // ishl_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // ushr_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm64, // sshr_imm // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // bitrev // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // clz // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // cls // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // ctz // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // bswap // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // popcnt // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::FloatCompare, // fcmp // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fadd // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fsub // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fmul // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fdiv // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // sqrt // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Ternary, // fma // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fneg // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fabs // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fcopysign // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fmin // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // fmax // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // ceil // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // floor // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // trunc // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // nearest // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::LoadNoOffset, // bitcast // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // scalar_to_vector // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // bmask // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // ireduce // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // snarrow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // unarrow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // uunarrow // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // swiden_low // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // swiden_high // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // uwiden_low // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // uwiden_high // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // iadd_pairwise // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // x86_pmaddubsw // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // uextend // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // sextend // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fpromote // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fdemote // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fvdemote // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fvpromote_low // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_to_uint // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_to_sint // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_to_uint_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_to_sint_sat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // x86_cvtt2dq // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_from_uint // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // fcvt_from_sint // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Unary, // isplit // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::Binary, // iconcat // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::AtomicRmw, // atomic_rmw // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::AtomicCas, // atomic_cas // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::LoadNoOffset, // atomic_load // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::StoreNoOffset, // atomic_store // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::NullAry, // fence // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
    InstructionFormat::BinaryImm8, // extract_vector // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:667
]; // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:675

fn opcode_name(opc: Opcode) -> &'static str {
    match opc {
        Opcode::AtomicCas => {
            "atomic_cas"
        }
        Opcode::AtomicLoad => {
            "atomic_load"
        }
        Opcode::AtomicRmw => {
            "atomic_rmw"
        }
        Opcode::AtomicStore => {
            "atomic_store"
        }
        Opcode::AvgRound => {
            "avg_round"
        }
        Opcode::Band => {
            "band"
        }
        Opcode::BandImm => {
            "band_imm"
        }
        Opcode::BandNot => {
            "band_not"
        }
        Opcode::Bitcast => {
            "bitcast"
        }
        Opcode::Bitrev => {
            "bitrev"
        }
        Opcode::Bitselect => {
            "bitselect"
        }
        Opcode::Bmask => {
            "bmask"
        }
        Opcode::Bnot => {
            "bnot"
        }
        Opcode::Bor => {
            "bor"
        }
        Opcode::BorImm => {
            "bor_imm"
        }
        Opcode::BorNot => {
            "bor_not"
        }
        Opcode::BrTable => {
            "br_table"
        }
        Opcode::Brif => {
            "brif"
        }
        Opcode::Bswap => {
            "bswap"
        }
        Opcode::Bxor => {
            "bxor"
        }
        Opcode::BxorImm => {
            "bxor_imm"
        }
        Opcode::BxorNot => {
            "bxor_not"
        }
        Opcode::Call => {
            "call"
        }
        Opcode::CallIndirect => {
            "call_indirect"
        }
        Opcode::Ceil => {
            "ceil"
        }
        Opcode::Cls => {
            "cls"
        }
        Opcode::Clz => {
            "clz"
        }
        Opcode::Ctz => {
            "ctz"
        }
        Opcode::Debugtrap => {
            "debugtrap"
        }
        Opcode::DynamicStackAddr => {
            "dynamic_stack_addr"
        }
        Opcode::DynamicStackLoad => {
            "dynamic_stack_load"
        }
        Opcode::DynamicStackStore => {
            "dynamic_stack_store"
        }
        Opcode::ExtractVector => {
            "extract_vector"
        }
        Opcode::Extractlane => {
            "extractlane"
        }
        Opcode::F128const => {
            "f128const"
        }
        Opcode::F16const => {
            "f16const"
        }
        Opcode::F32const => {
            "f32const"
        }
        Opcode::F64const => {
            "f64const"
        }
        Opcode::Fabs => {
            "fabs"
        }
        Opcode::Fadd => {
            "fadd"
        }
        Opcode::Fcmp => {
            "fcmp"
        }
        Opcode::Fcopysign => {
            "fcopysign"
        }
        Opcode::FcvtFromSint => {
            "fcvt_from_sint"
        }
        Opcode::FcvtFromUint => {
            "fcvt_from_uint"
        }
        Opcode::FcvtToSint => {
            "fcvt_to_sint"
        }
        Opcode::FcvtToSintSat => {
            "fcvt_to_sint_sat"
        }
        Opcode::FcvtToUint => {
            "fcvt_to_uint"
        }
        Opcode::FcvtToUintSat => {
            "fcvt_to_uint_sat"
        }
        Opcode::Fdemote => {
            "fdemote"
        }
        Opcode::Fdiv => {
            "fdiv"
        }
        Opcode::Fence => {
            "fence"
        }
        Opcode::Floor => {
            "floor"
        }
        Opcode::Fma => {
            "fma"
        }
        Opcode::Fmax => {
            "fmax"
        }
        Opcode::Fmin => {
            "fmin"
        }
        Opcode::Fmul => {
            "fmul"
        }
        Opcode::Fneg => {
            "fneg"
        }
        Opcode::Fpromote => {
            "fpromote"
        }
        Opcode::Fsub => {
            "fsub"
        }
        Opcode::FuncAddr => {
            "func_addr"
        }
        Opcode::Fvdemote => {
            "fvdemote"
        }
        Opcode::FvpromoteLow => {
            "fvpromote_low"
        }
        Opcode::GetFramePointer => {
            "get_frame_pointer"
        }
        Opcode::GetPinnedReg => {
            "get_pinned_reg"
        }
        Opcode::GetReturnAddress => {
            "get_return_address"
        }
        Opcode::GetStackPointer => {
            "get_stack_pointer"
        }
        Opcode::GlobalValue => {
            "global_value"
        }
        Opcode::Iabs => {
            "iabs"
        }
        Opcode::Iadd => {
            "iadd"
        }
        Opcode::IaddImm => {
            "iadd_imm"
        }
        Opcode::IaddPairwise => {
            "iadd_pairwise"
        }
        Opcode::Icmp => {
            "icmp"
        }
        Opcode::IcmpImm => {
            "icmp_imm"
        }
        Opcode::Iconcat => {
            "iconcat"
        }
        Opcode::Iconst => {
            "iconst"
        }
        Opcode::Imul => {
            "imul"
        }
        Opcode::ImulImm => {
            "imul_imm"
        }
        Opcode::Ineg => {
            "ineg"
        }
        Opcode::Insertlane => {
            "insertlane"
        }
        Opcode::Ireduce => {
            "ireduce"
        }
        Opcode::IrsubImm => {
            "irsub_imm"
        }
        Opcode::Ishl => {
            "ishl"
        }
        Opcode::IshlImm => {
            "ishl_imm"
        }
        Opcode::Isplit => {
            "isplit"
        }
        Opcode::Istore16 => {
            "istore16"
        }
        Opcode::Istore32 => {
            "istore32"
        }
        Opcode::Istore8 => {
            "istore8"
        }
        Opcode::Isub => {
            "isub"
        }
        Opcode::Jump => {
            "jump"
        }
        Opcode::Load => {
            "load"
        }
        Opcode::Nearest => {
            "nearest"
        }
        Opcode::Nop => {
            "nop"
        }
        Opcode::Popcnt => {
            "popcnt"
        }
        Opcode::Return => {
            "return"
        }
        Opcode::ReturnCall => {
            "return_call"
        }
        Opcode::ReturnCallIndirect => {
            "return_call_indirect"
        }
        Opcode::Rotl => {
            "rotl"
        }
        Opcode::RotlImm => {
            "rotl_imm"
        }
        Opcode::Rotr => {
            "rotr"
        }
        Opcode::RotrImm => {
            "rotr_imm"
        }
        Opcode::SaddOverflow => {
            "sadd_overflow"
        }
        Opcode::SaddOverflowCin => {
            "sadd_overflow_cin"
        }
        Opcode::SaddSat => {
            "sadd_sat"
        }
        Opcode::ScalarToVector => {
            "scalar_to_vector"
        }
        Opcode::Sdiv => {
            "sdiv"
        }
        Opcode::SdivImm => {
            "sdiv_imm"
        }
        Opcode::Select => {
            "select"
        }
        Opcode::SelectSpectreGuard => {
            "select_spectre_guard"
        }
        Opcode::SetPinnedReg => {
            "set_pinned_reg"
        }
        Opcode::Sextend => {
            "sextend"
        }
        Opcode::Shuffle => {
            "shuffle"
        }
        Opcode::Sload16 => {
            "sload16"
        }
        Opcode::Sload16x4 => {
            "sload16x4"
        }
        Opcode::Sload32 => {
            "sload32"
        }
        Opcode::Sload32x2 => {
            "sload32x2"
        }
        Opcode::Sload8 => {
            "sload8"
        }
        Opcode::Sload8x8 => {
            "sload8x8"
        }
        Opcode::Smax => {
            "smax"
        }
        Opcode::Smin => {
            "smin"
        }
        Opcode::SmulOverflow => {
            "smul_overflow"
        }
        Opcode::Smulhi => {
            "smulhi"
        }
        Opcode::Snarrow => {
            "snarrow"
        }
        Opcode::Splat => {
            "splat"
        }
        Opcode::SqmulRoundSat => {
            "sqmul_round_sat"
        }
        Opcode::Sqrt => {
            "sqrt"
        }
        Opcode::Srem => {
            "srem"
        }
        Opcode::SremImm => {
            "srem_imm"
        }
        Opcode::Sshr => {
            "sshr"
        }
        Opcode::SshrImm => {
            "sshr_imm"
        }
        Opcode::SsubOverflow => {
            "ssub_overflow"
        }
        Opcode::SsubOverflowBin => {
            "ssub_overflow_bin"
        }
        Opcode::SsubSat => {
            "ssub_sat"
        }
        Opcode::StackAddr => {
            "stack_addr"
        }
        Opcode::StackLoad => {
            "stack_load"
        }
        Opcode::StackStore => {
            "stack_store"
        }
        Opcode::StackSwitch => {
            "stack_switch"
        }
        Opcode::Store => {
            "store"
        }
        Opcode::SwidenHigh => {
            "swiden_high"
        }
        Opcode::SwidenLow => {
            "swiden_low"
        }
        Opcode::Swizzle => {
            "swizzle"
        }
        Opcode::SymbolValue => {
            "symbol_value"
        }
        Opcode::TlsValue => {
            "tls_value"
        }
        Opcode::Trap => {
            "trap"
        }
        Opcode::Trapnz => {
            "trapnz"
        }
        Opcode::Trapz => {
            "trapz"
        }
        Opcode::Trunc => {
            "trunc"
        }
        Opcode::TryCall => {
            "try_call"
        }
        Opcode::TryCallIndirect => {
            "try_call_indirect"
        }
        Opcode::UaddOverflow => {
            "uadd_overflow"
        }
        Opcode::UaddOverflowCin => {
            "uadd_overflow_cin"
        }
        Opcode::UaddOverflowTrap => {
            "uadd_overflow_trap"
        }
        Opcode::UaddSat => {
            "uadd_sat"
        }
        Opcode::Udiv => {
            "udiv"
        }
        Opcode::UdivImm => {
            "udiv_imm"
        }
        Opcode::Uextend => {
            "uextend"
        }
        Opcode::Uload16 => {
            "uload16"
        }
        Opcode::Uload16x4 => {
            "uload16x4"
        }
        Opcode::Uload32 => {
            "uload32"
        }
        Opcode::Uload32x2 => {
            "uload32x2"
        }
        Opcode::Uload8 => {
            "uload8"
        }
        Opcode::Uload8x8 => {
            "uload8x8"
        }
        Opcode::Umax => {
            "umax"
        }
        Opcode::Umin => {
            "umin"
        }
        Opcode::UmulOverflow => {
            "umul_overflow"
        }
        Opcode::Umulhi => {
            "umulhi"
        }
        Opcode::Unarrow => {
            "unarrow"
        }
        Opcode::Urem => {
            "urem"
        }
        Opcode::UremImm => {
            "urem_imm"
        }
        Opcode::Ushr => {
            "ushr"
        }
        Opcode::UshrImm => {
            "ushr_imm"
        }
        Opcode::UsubOverflow => {
            "usub_overflow"
        }
        Opcode::UsubOverflowBin => {
            "usub_overflow_bin"
        }
        Opcode::UsubSat => {
            "usub_sat"
        }
        Opcode::Uunarrow => {
            "uunarrow"
        }
        Opcode::UwidenHigh => {
            "uwiden_high"
        }
        Opcode::UwidenLow => {
            "uwiden_low"
        }
        Opcode::VallTrue => {
            "vall_true"
        }
        Opcode::VanyTrue => {
            "vany_true"
        }
        Opcode::Vconst => {
            "vconst"
        }
        Opcode::VhighBits => {
            "vhigh_bits"
        }
        Opcode::X86Blendv => {
            "x86_blendv"
        }
        Opcode::X86Cvtt2dq => {
            "x86_cvtt2dq"
        }
        Opcode::X86Pmaddubsw => {
            "x86_pmaddubsw"
        }
        Opcode::X86Pmulhrsw => {
            "x86_pmulhrsw"
        }
        Opcode::X86Pshufb => {
            "x86_pshufb"
        }
    }
}

const OPCODE_HASH_TABLE: [Option<Opcode>; 256] = [ // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:696
    Some(Opcode::Imul), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::TlsValue), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Brif), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Nearest), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtToSintSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fsub), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Trunc), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Urem), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Iconst), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::ReturnCall), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Umin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Store), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::GetFramePointer), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UshrImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Isub), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtFromSint), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Trap), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sdiv), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Srem), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SshrImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uunarrow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UaddOverflowCin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bxor), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::X86Pmaddubsw), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Umax), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SremImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Insertlane), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BxorNot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Swizzle), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Load), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fadd), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Jump), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BxorImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Shuffle), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fneg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Umulhi), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Ushr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::UaddOverflowTrap), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtFromUint), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::VallTrue), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Band), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::SsubOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uload16x4), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Ishl), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fmax), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Vconst), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Call), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::ExtractVector), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sqrt), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Ceil), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Ineg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FuncAddr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SaddSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Popcnt), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Fabs), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fmin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SsubOverflowBin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::GlobalValue), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bnot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sextend), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Isplit), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtToUint), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::RotlImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fcmp), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SwidenHigh), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fmul), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtToSint), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::UsubOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uload8x8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Fdiv), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::UremImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::AtomicLoad), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Trapnz), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uload16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::IaddImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uload32), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bitrev), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Smulhi), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::TryCall), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::BorNot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Sload8x8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::X86Blendv), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SetPinnedReg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::ImulImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Ireduce), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::RotrImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::DynamicStackStore), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::StackStore), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UwidenLow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Select), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BorImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Istore32), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FvpromoteLow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Istore16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Fdemote), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::IcmpImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fvdemote), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Sload16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fcopysign), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::SdivImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Unarrow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::AvgRound), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sload32), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::X86Pshufb), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Extractlane), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::StackAddr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SaddOverflowCin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UaddOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BandImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Return), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Uload32x2), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::VanyTrue), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::UsubSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::DynamicStackLoad), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Iconcat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SmulOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Fence), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Fma), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bitselect), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Istore8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BrTable), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::F64const), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::StackSwitch), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::StackLoad), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::IrsubImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Nop), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SqmulRoundSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::X86Pmulhrsw), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Debugtrap), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sload16x4), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UmulOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::IshlImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SaddOverflow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Ctz), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bor), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::TryCallIndirect), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::BandNot), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Clz), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UwidenHigh), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Uextend), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Floor), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UaddSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sload32x2), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::SelectSpectreGuard), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Cls), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Fpromote), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bitcast), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::SymbolValue), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::DynamicStackAddr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bmask), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::GetPinnedReg), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SsubSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::AtomicRmw), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::ScalarToVector), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Uload8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::FcvtToUintSat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Smin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Trapz), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Iabs), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::F16const), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Udiv), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::AtomicCas), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::GetReturnAddress), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UsubOverflowBin), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::SwidenLow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::ReturnCallIndirect), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Rotl), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::IaddPairwise), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Smax), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::F128const), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::F32const), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::UdivImm), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Splat), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Rotr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Snarrow), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::CallIndirect), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Sload8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::X86Cvtt2dq), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::VhighBits), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Iadd), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Icmp), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::GetStackPointer), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::Bswap), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
    Some(Opcode::Sshr), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    Some(Opcode::AtomicStore), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:704
    None, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:705
]; // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:709


// Table of opcode constraints.
const OPCODE_CONSTRAINTS: [OpcodeConstraints; 185] = [ // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:830
    // Jump: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Brif: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BrTable: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Concrete(ir::types::I32)']
    OpcodeConstraints {
        flags: 0x20, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Debugtrap: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Trap: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Trapz: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Trapnz: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Return: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Call: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // CallIndirect: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // ReturnCall: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // ReturnCallIndirect: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FuncAddr: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // TryCall: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // TryCallIndirect: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Splat: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'LaneOf']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Swizzle: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)']
    OpcodeConstraints {
        flags: 0x41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // X86Pshufb: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)']
    OpcodeConstraints {
        flags: 0x41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Insertlane: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'LaneOf']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 9, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Extractlane: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['LaneOf', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 2, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Smin: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Umin: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Smax: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Umax: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // AvgRound: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UaddSat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SaddSat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UsubSat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SsubSat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Load: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Store: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload8: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload8: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Istore8: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload16: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload16: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Istore16: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload32: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I64)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload32: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I64)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Istore32: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['Concrete(ir::types::I64)', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={64})
    OpcodeConstraints {
        flags: 0x58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 7, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // StackSwitch: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload8x8: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I16X8)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 22, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload8x8: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I16X8)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 22, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload16x4: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I32X4)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 24, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload16x4: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I32X4)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 24, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uload32x2: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I64X2)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 26, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sload32x2: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I64X2)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 26, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // StackLoad: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // StackStore: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // StackAddr: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // DynamicStackLoad: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // DynamicStackStore: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // DynamicStackAddr: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // GlobalValue: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SymbolValue: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // TlsValue: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // GetPinnedReg: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SetPinnedReg: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x38, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // GetFramePointer: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // GetStackPointer: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // GetReturnAddress: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Iconst: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // F16const: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Concrete(ir::types::F16)']
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 28, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // F32const: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Concrete(ir::types::F32)']
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // F64const: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Concrete(ir::types::F64)']
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 30, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // F128const: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Concrete(ir::types::F128)']
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 31, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Vconst: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=['Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x01, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 9, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Shuffle: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)']
    OpcodeConstraints {
        flags: 0x41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 6, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Nop: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Select: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Free(0)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SelectSpectreGuard: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Free(0)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bitselect: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // X86Blendv: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // VanyTrue: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I8)', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 9, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 36, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // VallTrue: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I8)', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 9, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 36, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // VhighBits: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(9)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 37, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Icmp: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['AsTruthy', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // IcmpImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['Concrete(ir::types::I8)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 36, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Iadd: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Isub: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ineg: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Iabs: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Imul: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Umulhi: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Smulhi: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SqmulRoundSat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={4, 8}, ints={16, 32})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // X86Pmulhrsw: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={4, 8}, ints={16, 32})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Udiv: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sdiv: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Urem: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Srem: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // IaddImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // ImulImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UdivImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SdivImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UremImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SremImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // IrsubImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SaddOverflowCin: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same', 'Concrete(ir::types::I8)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x6a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UaddOverflowCin: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same', 'Concrete(ir::types::I8)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x6a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UaddOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SaddOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UsubOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SsubOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UmulOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SmulOverflow: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64})
    OpcodeConstraints {
        flags: 0x4a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UaddOverflowTrap: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={32, 64})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 1, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SsubOverflowBin: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same', 'Concrete(ir::types::I8)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x6a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UsubOverflowBin: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Concrete(ir::types::I8)', 'Same', 'Same', 'Concrete(ir::types::I8)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x6a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Band: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bor: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bxor: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bnot: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BandNot: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BorNot: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BxorNot: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 10, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BandImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BorImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // BxorImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Rotl: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 46, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Rotr: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 46, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // RotlImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // RotrImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ishl: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 46, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ushr: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 46, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sshr: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 46, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // IshlImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UshrImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SshrImm: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bitrev: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Clz: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Cls: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ctz: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bswap: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 13, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Popcnt: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 11, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fcmp: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['AsTruthy', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fadd: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fsub: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fmul: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fdiv: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sqrt: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fma: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fneg: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fabs: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fcopysign: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fmin: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fmax: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ceil: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Floor: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Trunc: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Nearest: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Same']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x29, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 14, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bitcast: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(5)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 5, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // ScalarToVector: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'LaneOf']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 9, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 4, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Bmask: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(0)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 32, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Ireduce: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Wider']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 51, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Snarrow: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['SplitLanes', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 15, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 53, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Unarrow: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['SplitLanes', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 15, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 53, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uunarrow: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['SplitLanes', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8}, ints={16, 32, 64})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 15, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 53, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SwidenLow: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['MergeLanes', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 56, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // SwidenHigh: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['MergeLanes', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 56, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UwidenLow: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['MergeLanes', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 56, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // UwidenHigh: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['MergeLanes', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 56, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // IaddPairwise: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
    OpcodeConstraints {
        flags: 0x49, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 16, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // X86Pmaddubsw: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Concrete(ir::types::I16X8)', 'Concrete(ir::types::I8X16)', 'Concrete(ir::types::I8X16)']
    OpcodeConstraints {
        flags: 0x41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Uextend: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Narrower']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 61, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Sextend: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Narrower']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 61, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fpromote: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Narrower']
    // Polymorphic over TypeSet(lanes={1}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 17, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 61, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fdemote: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Wider']
    // Polymorphic over TypeSet(lanes={1}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 17, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 51, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fvdemote: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Concrete(ir::types::F32X4)', 'Concrete(ir::types::F64X2)']
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 63, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FvpromoteLow: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Concrete(ir::types::F64X2)', 'Concrete(ir::types::F32X4)']
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 64, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtToUint: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(17)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 66, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtToSint: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(17)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 66, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtToUintSat: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(14)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 68, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtToSintSat: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(14)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 68, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // X86Cvtt2dq: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(14)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 3, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 68, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtFromUint: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(3)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 70, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // FcvtFromSint: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(3)']
    // Polymorphic over TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 18, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 70, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Isplit: fixed_results=2, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['HalfWidth', 'HalfWidth', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x3a, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 13, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 72, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Iconcat: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['DoubleWidth', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64})
    OpcodeConstraints {
        flags: 0x59, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 8, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 75, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // AtomicRmw: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=2
    // Constraints=['Same', 'Free(1)', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x41, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 77, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // AtomicCas: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=false, fixed_values=3
    // Constraints=['Same', 'Free(1)', 'Same', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x69, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 77, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // AtomicLoad: fixed_results=1, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=1
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x21, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // AtomicStore: fixed_results=0, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=2
    // Constraints=['Same', 'Free(1)']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x58, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 12, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // Fence: fixed_results=0, use_typevar_operand=false, requires_typevar_operand=false, fixed_values=0
    // Constraints=[]
    OpcodeConstraints {
        flags: 0x00, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 255, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 0, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
    // ExtractVector: fixed_results=1, use_typevar_operand=true, requires_typevar_operand=true, fixed_values=1
    // Constraints=['DynamicToVector', 'Same']
    // Polymorphic over TypeSet(lanes={1}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
    OpcodeConstraints {
        flags: 0x39, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:901
        typeset_offset: 19, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:902
        constraint_offset: 81, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:903
    }
    ,
]; // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:908

// Table of value type sets.
const TYPE_SETS: [ir::instructions::ValueTypeSet; 20] = [ // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:791
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={32, 64})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(96), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={16, 32, 64})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(112), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={64})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(64), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={8, 16, 32, 64})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(120), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={4, 8}, ints={16, 32})
        lanes: ScalarBitSet::<u16>(12), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(48), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={2, 4, 8}, ints={16, 32, 64})
        lanes: ScalarBitSet::<u16>(14), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(14), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(112), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={2, 4, 8, 16}, ints={8, 16, 32})
        lanes: ScalarBitSet::<u16>(30), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(30), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(56), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(511), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={8, 16, 32, 64, 128}, floats={16, 32, 64, 128})
        lanes: ScalarBitSet::<u16>(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        dynamic_lanes: ScalarBitSet::<u16>(510), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        ints: ScalarBitSet::<u8>(248), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
        floats: ScalarBitSet::<u8>(240), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:759
    }
    ,
]; // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:808

// Table of operand constraint sequences.
const OPERAND_CONSTRAINTS: [OperandConstraint; 83] = [ // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:915
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I32), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::LaneOf, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8X16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8X16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8X16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::LaneOf, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I64), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I64), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I16X8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I32X4), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I64X2), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F32), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F64), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F128), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(9), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::AsTruthy, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(0), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(5), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Wider, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::SplitLanes, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::MergeLanes, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I16X8), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8X16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::I8X16), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Narrower, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F32X4), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F64X2), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Concrete(ir::types::F32X4), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(17), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(14), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(3), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::HalfWidth, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::HalfWidth, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::DoubleWidth, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Free(1), // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::DynamicToVector, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
    OperandConstraint::Same, // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:922
]; // /home/freddy/.accounts/me1ns/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.123.19/src/gen_inst.rs:925
