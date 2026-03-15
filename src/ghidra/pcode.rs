use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use crate::{Error, Result};

#[derive(FromPrimitive, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PcodeOp {
    /// Place holder for unimplemented instruction
    Unimplemented = 0,
    /// Copy one operand to another
    Copy = 1,
    /// Dereference a pointer into specified space
    Load = 2,
    /// Store at a pointer into specified space
    Store = 3,

    /// Always branch 
    Branch = 4,
    /// Conditional branch 
    Cbranch = 5,
    /// An indirect branch (jumptable)
    Branchind = 6,

    /// A call with absolute address
    Call = 7,
    /// An indirect call
    Callind = 8,		
    /// Other unusual subroutine calling conventions
    Callother = 9,
    /// A return from subroutine
    Return = 10,

    /// Return TRUE if operand1 == operand2 
    IntEqual = 11,
    /// Return TRUE if operand1 != operand2
    IntNotequal = 12,
    /// Return TRUE if signed op1 < signed op2
    IntSless = 13,
    /// Return TRUE if signed op1 <= signed op2
    IntSlessequal = 14,
    /// Return TRUE if unsigned op1 < unsigned op2
    IntLess = 15,
    // Also indicates borrow on unsigned subtraction
    /// Return TRUE if unsigned op1 <= unsigned op2
    IntLessequal = 16,
    /// Zero extend operand 
    IntZext = 17,
    /// Sign extend operand 
    IntSext = 18,
    /// Unsigned addition of operands of same size 
    IntAdd = 19,
    /// Unsigned subtraction of operands of same size 
    IntSub = 20,
    /// TRUE if adding two operands has overflow (carry) 
    IntCarry = 21,
	/// TRUE if carry in signed addition of 2 ops 
    IntScarry = 22,
	/// TRUE if borrow in signed subtraction of 2 ops 
    IntSborrow = 23,
	/// Twos complement (for subtracting) of operand 
    Int2comp = 24,
	IntNegate = 25,
	/// Exclusive OR of two operands of same size 
    IntXor = 26,
	IntAnd = 27,
	IntOr = 28,
	/// Left shift 
    IntLeft = 29,
    /// Right shift zero fill 
    IntRight = 30,
	/// Signed right shift 
    IntSright = 31,
	/// Integer multiplication 
    IntMult = 32,
	/// Unsigned integer division
    IntDiv = 33,
	/// Signed integer division
    IntSdiv = 34,
	/// Unsigned mod (remainder)
    IntRem = 35,
	/// Signed mod (remainder)
    IntSrem = 36,

	/// Boolean negate or not
    BoolNegate = 37,
	/// Boolean xor
    BoolXor = 38,
	/// Boolean and (&&)
    BoolAnd = 39,
	/// Boolean or (||)
    BoolOr = 40,

	// floating point instructions:  No floating point data format is specified here,
	// although the exact operation of these instructions obviously depends on the
	// format.  For simulation, a "mode" variable specifying the floating point format
	// will be necessary.
	/// Return TRUE if operand1 == operand2    
    FloatEqual = 41,
	/// Return TRUE if operand1 != operand2    
    FloatNotequal = 42,
	/// Return TRUE if op1 < op2 
    FloatLess = 43,
	/// Return TRUE if op1 <= op2
    FloatLessequal = 44,
	// Slot 45 is unused
	/// Return TRUE if neither op1 is NaN 
    FloatNan = 46,

	/// float addition
    FloatAdd = 47,
	/// float division
    FloatDiv = 48,
	/// float multiplication
    FloatMult = 49,
	/// float subtraction
    FloatSub = 50,
	/// float negation
    FloatNeg = 51,
	/// float absolute value
    FloatAbs = 52,
	/// float square root
    FloatSqrt = 53,

	/// convert int type to float type
    #[serde(alias = "INT2FLOAT")]
    FloatInt2float = 54,
	/// convert between float sizes
    #[serde(alias = "FLOAT2FLOAT")]
    FloatFloat2float = 55,
	/// round towards zero
    FloatTrunc = 56,
	/// round towards +infinity
    FloatCeil = 57,
	/// round towards -infinity
    FloatFloor = 58,
	/// round towards nearest
    FloatRound = 59,

	// Internal opcodes for simplification.  Not typically generated in direct
	// translation.
	/// Output equal to one of inputs, depending on execution
    Multiequal = 60,
	/// Output probably equals input, but may be indirectly affected
    Indirect = 61,
	/// Output is constructed from multiple pieces
    Piece = 62,
	/// Output is a subpiece of input0, input1=offset into input0
    Subpiece = 63,

	/// Cast from one type to another
    Cast = 64,
	/// outptr = ptrbase,offset, (size multiplier)
    Ptradd = 65,
	/// outptr = &(ptr->subfield)
    Ptrsub = 66,      
	Segmentop = 67,
	Cpoolref = 68,
	New = 69,
	Insert = 70,
	Zpull = 71,
	Popcount = 72,
    Lzcount = 73,
    Spull = 74,

	PcodeMax = 75,
}

impl PcodeOp {
    pub fn mnemonic(&self) -> &str {
        match self {
            PcodeOp::Unimplemented => "UNIMPLEMENTED",
            PcodeOp::Copy => "COPY",
            PcodeOp::Load => "LOAD",
            PcodeOp::Store => "STORE",
            PcodeOp::Branch => "BRANCH",
            PcodeOp::Cbranch => "CBRANCH",
            PcodeOp::Branchind => "BRANCHIND",
            PcodeOp::Call => "CALL",
            PcodeOp::Callind => "CALLIND",
            PcodeOp::Callother => "CALLOTHER",
            PcodeOp::Return => "RETURN",
            PcodeOp::IntEqual => "INT_EQUAL",
            PcodeOp::IntNotequal => "INT_NOTEQUAL",
            PcodeOp::IntSless => "INT_SLESS",
            PcodeOp::IntSlessequal => "INT_SLESSEQUAL",
            PcodeOp::IntLess => "INT_LESS",
            PcodeOp::IntLessequal => "INT_LESSEQUAL",
            PcodeOp::IntZext => "INT_ZEXT",
            PcodeOp::IntSext => "INT_SEXT",
            PcodeOp::IntAdd => "INT_ADD",
            PcodeOp::IntSub => "INT_SUB",
            PcodeOp::IntCarry => "INT_CARRY",
            PcodeOp::IntScarry => "INT_SCARRY",
            PcodeOp::IntSborrow => "INT_SBORROW",
            PcodeOp::Int2comp => "INT_2COMP",
            PcodeOp::IntNegate => "INT_NEGATE",
            PcodeOp::IntXor => "INT_XOR",
            PcodeOp::IntAnd => "INT_AND",
            PcodeOp::IntOr => "INT_OR",
            PcodeOp::IntLeft => "INT_LEFT",
            PcodeOp::IntRight => "INT_RIGHT",
            PcodeOp::IntSright => "INT_SRIGHT",
            PcodeOp::IntMult => "INT_MULT",
            PcodeOp::IntDiv => "INT_DIV",
            PcodeOp::IntSdiv => "INT_SDIV",
            PcodeOp::IntRem => "INT_REM",
            PcodeOp::IntSrem => "INT_SREM",
            PcodeOp::BoolNegate => "BOOL_NEGATE",
            PcodeOp::BoolXor => "BOOL_XOR",
            PcodeOp::BoolAnd => "BOOL_AND",
            PcodeOp::BoolOr => "BOOL_OR",
            PcodeOp::FloatEqual => "FLOAT_EQUAL",
            PcodeOp::FloatNotequal => "FLOAT_NOTEQUAL",
            PcodeOp::FloatLess => "FLOAT_LESS",
            PcodeOp::FloatLessequal => "FLOAT_LESSEQUAL",
            PcodeOp::FloatNan => "FLOAT_NAN",
            PcodeOp::FloatAdd => "FLOAT_ADD",
            PcodeOp::FloatDiv => "FLOAT_DIV",
            PcodeOp::FloatMult => "FLOAT_MULT",
            PcodeOp::FloatSub => "FLOAT_SUB",
            PcodeOp::FloatNeg => "FLOAT_NEG",
            PcodeOp::FloatAbs => "FLOAT_ABS",
            PcodeOp::FloatSqrt => "FLOAT_SQRT",
            PcodeOp::FloatInt2float => "FLOAT_INT2FLOAT",
            PcodeOp::FloatFloat2float => "FLOAT_FLOAT2FLOAT",
            PcodeOp::FloatTrunc => "FLOAT_TRUNC",
            PcodeOp::FloatCeil => "FLOAT_CEIL",
            PcodeOp::FloatFloor => "FLOAT_FLOOR",
            PcodeOp::FloatRound => "FLOAT_ROUND",
            PcodeOp::Multiequal => "MULTIEQUAL",
            PcodeOp::Indirect => "INDIRECT",
            PcodeOp::Piece => "PIECE",
            PcodeOp::Subpiece => "SUBPIECE",
            PcodeOp::Cast => "CAST",
            PcodeOp::Ptradd => "PTRADD",
            PcodeOp::Ptrsub => "PTRSUB",
            PcodeOp::Segmentop => "SEGMENTOP",
            PcodeOp::Cpoolref => "CPOOLREF",
            PcodeOp::New => "NEW",
            PcodeOp::Insert => "INSERT",
            PcodeOp::Zpull => "ZPULL",
            PcodeOp::Popcount => "POPCOUNT",
            PcodeOp::Lzcount => "LZCOUNT",
            PcodeOp::Spull => "SPULL",
            PcodeOp::PcodeMax => "PCODE_MAX",
        }
    }
}

impl TryFrom<&str> for PcodeOp {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        value.to_string().try_into()
    }
}

impl TryFrom<String> for PcodeOp {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let value = value.to_uppercase();
        let pcodes = vec![
            "UNIMPLEMENTED", "COPY", "LOAD", "STORE", "BRANCH", 
            "CBRANCH", "BRANCHIND", "CALL", "CALLIND", "CALLOTHER",
            "RETURN", "INT_EQUAL", "INT_NOTEQUAL", "INT_SLESS", "INT_SLESSEQUAL",
            "INT_LESS", "INT_LESSEQUAL", "INT_ZEXT", "INT_SEXT", "INT_ADD",
            "INT_SUB", "INT_CARRY", "INT_SCARRY", "INT_SBORROW", "INT_2COMP",
            "INT_NEGATE", "INT_XOR", "INT_AND", "INT_OR", "INT_LEFT",
            "INT_RIGHT", "INT_SRIGHT", "INT_MULT", "INT_DIV", "INT_SDIV",
            "INT_REM", "INT_SREM", "BOOL_NEGATE", "BOOL_XOR", "BOOL_AND",
            "BOOL_OR", "FLOAT_EQUAL", "FLOAT_NOTEQUAL", "FLOAT_LESS", "FLOAT_LESSEQUAL",
            "UNUSED", // 45 is unused
            "FLOAT_NAN","FLOAT_ADD","FLOAT_DIV","FLOAT_MULT",
            "FLOAT_SUB","FLOAT_NEG", "FLOAT_ABS","FLOAT_SQRT","FLOAT_INT2FLOAT",
            "FLOAT_FLOAT2FLOAT", "FLOAT_TRUNC","FLOAT_CEIL","FLOAT_FLOOR","FLOAT_ROUND",
            "MULTIEQUAL","INDIRECT","PIECE","SUBPIECE", "CAST",
            "PTRADD","PTRSUB","SEGMENTOP","CPOOLREF","NEW",
            "INSERT", "ZPULL","POPCOUNT","LZCOUNT","SPULL",
            "PCODE_MAX" // 75 is PCODE_MAX
        ];
        if let Some(code) = pcodes.iter().position(|&m| m == value) {
            if code != 45 {
                PcodeOp::from_u32(code as u32)
                    .ok_or_else(|| Error::InvalidPcode(code as u32))
            } else {
                Err(Error::InvalidPcode(code as u32))
            }
        } else {
            Err(Error::Parse(format!("Invalid pcode mnemonic: {}", value)))
        }
    }
}
