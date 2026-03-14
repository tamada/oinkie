use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use crate::{Error, Result};

#[derive(FromPrimitive, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
#[serde(rename_all = "UPPERCASE")]
pub enum PcodeOp {
    /// Place holder for unimplemented instruction
    UNIMPLEMENTED = 0,
    /// Copy one operand to another
    COPY = 1,
    /// Dereference a pointer into specified space
    LOAD = 2,
    /// Store at a pointer into specified space
    STORE = 3,

    /// Always branch 
    BRANCH = 4,
    /// Conditional branch 
    CBRANCH = 5,
    /// An indirect branch (jumptable)
    BRANCHIND = 6,

    /// A call with absolute address
    CALL = 7,
    /// An indirect call
    CALLIND = 8,		
    /// Other unusual subroutine calling conventions
    CALLOTHER = 9,
    /// A return from subroutine
    RETURN = 10,

    /// Return TRUE if operand1 == operand2 
    INT_EQUAL = 11,
    /// Return TRUE if operand1 != operand2
    INT_NOTEQUAL = 12,
    /// Return TRUE if signed op1 < signed op2
    INT_SLESS = 13,
    /// Return TRUE if signed op1 <= signed op2
    INT_SLESSEQUAL = 14,
    /// Return TRUE if unsigned op1 < unsigned op2
    INT_LESS = 15,
    // Also indicates borrow on unsigned subtraction
    /// Return TRUE if unsigned op1 <= unsigned op2
    INT_LESSEQUAL = 16,
    /// Zero extend operand 
    INT_ZEXT = 17,
    /// Sign extend operand 
    INT_SEXT = 18,
    /// Unsigned addition of operands of same size 
    INT_ADD = 19,
    /// Unsigned subtraction of operands of same size 
    INT_SUB = 20,
    /// TRUE if adding two operands has overflow (carry) 
    INT_CARRY = 21,
	/// TRUE if carry in signed addition of 2 ops 
    INT_SCARRY = 22,
	/// TRUE if borrow in signed subtraction of 2 ops 
    INT_SBORROW = 23,
	/// Twos complement (for subtracting) of operand 
    INT_2COMP = 24,
	INT_NEGATE = 25,
	/// Exclusive OR of two operands of same size 
    INT_XOR = 26,
	INT_AND = 27,
	INT_OR = 28,
	/// Left shift 
    INT_LEFT = 29,
    /// Right shift zero fill 
    INT_RIGHT = 30,
	/// Signed right shift 
    INT_SRIGHT = 31,
	/// Integer multiplication 
    INT_MULT = 32,
	/// Unsigned integer division
    INT_DIV = 33,
	/// Signed integer division
    INT_SDIV = 34,
	/// Unsigned mod (remainder)
    INT_REM = 35,
	/// Signed mod (remainder)
    INT_SREM = 36,

	/// Boolean negate or not
    BOOL_NEGATE = 37,
	/// Boolean xor
    BOOL_XOR = 38,
	/// Boolean and (&&)
    BOOL_AND = 39,
	/// Boolean or (||)
    BOOL_OR = 40,

	// floating point instructions:  No floating point data format is specified here,
	// although the exact operation of these instructions obviously depends on the
	// format.  For simulation, a "mode" variable specifying the floating point format
	// will be necessary.
	/// Return TRUE if operand1 == operand2    
    FLOAT_EQUAL = 41,
	/// Return TRUE if operand1 != operand2    
    FLOAT_NOTEQUAL = 42,
	/// Return TRUE if op1 < op2 
    FLOAT_LESS = 43,
	/// Return TRUE if op1 <= op2
    FLOAT_LESSEQUAL = 44,
	// Slot 45 is unused
	/// Return TRUE if neither op1 is NaN 
    FLOAT_NAN = 46,

	/// float addition
    FLOAT_ADD = 47,
	/// float division
    FLOAT_DIV = 48,
	/// float multiplication
    FLOAT_MULT = 49,
	/// float subtraction
    FLOAT_SUB = 50,
	/// float negation
    FLOAT_NEG = 51,
	/// float absolute value
    FLOAT_ABS = 52,
	/// float square root
    FLOAT_SQRT = 53,

	/// convert int type to float type
    #[serde(alias = "INT2FLOAT")]
    FLOAT_INT2FLOAT = 54,
	/// convert between float sizes
    #[serde(alias = "FLOAT2FLOAT")]
    FLOAT_FLOAT2FLOAT = 55,
	/// round towards zero
    FLOAT_TRUNC = 56,
	/// round towards +infinity
    FLOAT_CEIL = 57,
	/// round towards -infinity
    FLOAT_FLOOR = 58,
	/// round towards nearest
    FLOAT_ROUND = 59,

	// Internal opcodes for simplification.  Not typically generated in direct
	// translation.
	/// Output equal to one of inputs, depending on execution
    MULTIEQUAL = 60,
	/// Output probably equals input, but may be indirectly affected
    INDIRECT = 61,
	/// Output is constructed from multiple pieces
    PIECE = 62,
	/// Output is a subpiece of input0, input1=offset into input0
    SUBPIECE = 63,

	/// Cast from one type to another
    CAST = 64,
	/// outptr = ptrbase,offset, (size multiplier)
    PTRADD = 65,
	/// outptr = &(ptr->subfield)
    PTRSUB = 66,      
	SEGMENTOP = 67,
	CPOOLREF = 68,
	NEW = 69,
	INSERT = 70,
	ZPULL = 71,
	POPCOUNT = 72,
	LZCOUNT = 73,
	SPULL = 74,

	PCODE_MAX = 75,
}

impl PcodeOp {
    pub fn mnemonic(&self) -> &str {
        match self {
            PcodeOp::UNIMPLEMENTED => "UNIMPLEMENTED",
            PcodeOp::COPY => "COPY",
            PcodeOp::LOAD => "LOAD",
            PcodeOp::STORE => "STORE",
            PcodeOp::BRANCH => "BRANCH",
            PcodeOp::CBRANCH => "CBRANCH",
            PcodeOp::BRANCHIND => "BRANCHIND",
            PcodeOp::CALL => "CALL",
            PcodeOp::CALLIND => "CALLIND",
            PcodeOp::CALLOTHER => "CALLOTHER",
            PcodeOp::RETURN => "RETURN",
            PcodeOp::INT_EQUAL => "INT_EQUAL",
            PcodeOp::INT_NOTEQUAL => "INT_NOTEQUAL",
            PcodeOp::INT_SLESS => "INT_SLESS",
            PcodeOp::INT_SLESSEQUAL => "INT_SLESSEQUAL",
            PcodeOp::INT_LESS => "INT_LESS",
            PcodeOp::INT_LESSEQUAL => "INT_LESSEQUAL",
            PcodeOp::INT_ZEXT => "INT_ZEXT",
            PcodeOp::INT_SEXT => "INT_SEXT",
            PcodeOp::INT_ADD => "INT_ADD",
            PcodeOp::INT_SUB => "INT_SUB",
            PcodeOp::INT_CARRY => "INT_CARRY",
            PcodeOp::INT_SCARRY => "INT_SCARRY",
            PcodeOp::INT_SBORROW => "INT_SBORROW",
            PcodeOp::INT_2COMP => "INT_2COMP",
            PcodeOp::INT_NEGATE => "INT_NEGATE",
            PcodeOp::INT_XOR => "INT_XOR",
            PcodeOp::INT_AND => "INT_AND",
            PcodeOp::INT_OR => "INT_OR",
            PcodeOp::INT_LEFT => "INT_LEFT",
            PcodeOp::INT_RIGHT => "INT_RIGHT",
            PcodeOp::INT_SRIGHT => "INT_SRIGHT",
            PcodeOp::INT_MULT => "INT_MULT",
            PcodeOp::INT_DIV => "INT_DIV",
            PcodeOp::INT_SDIV => "INT_SDIV",
            PcodeOp::INT_REM => "INT_REM",
            PcodeOp::INT_SREM => "INT_SREM",
            PcodeOp::BOOL_NEGATE => "BOOL_NEGATE",
            PcodeOp::BOOL_XOR => "BOOL_XOR",
            PcodeOp::BOOL_AND => "BOOL_AND",
            PcodeOp::BOOL_OR => "BOOL_OR",
            PcodeOp::FLOAT_EQUAL => "FLOAT_EQUAL",
            PcodeOp::FLOAT_NOTEQUAL => "FLOAT_NOTEQUAL",
            PcodeOp::FLOAT_LESS => "FLOAT_LESS",
            PcodeOp::FLOAT_LESSEQUAL => "FLOAT_LESSEQUAL",
            PcodeOp::FLOAT_NAN => "FLOAT_NAN",
            PcodeOp::FLOAT_ADD => "FLOAT_ADD",
            PcodeOp::FLOAT_DIV => "FLOAT_DIV",
            PcodeOp::FLOAT_MULT => "FLOAT_MULT",
            PcodeOp::FLOAT_SUB => "FLOAT_SUB",
            PcodeOp::FLOAT_NEG => "FLOAT_NEG",
            PcodeOp::FLOAT_ABS => "FLOAT_ABS",
            PcodeOp::FLOAT_SQRT => "FLOAT_SQRT",
            PcodeOp::FLOAT_INT2FLOAT => "FLOAT_INT2FLOAT",
            PcodeOp::FLOAT_FLOAT2FLOAT => "FLOAT_FLOAT2FLOAT",
            PcodeOp::FLOAT_TRUNC => "FLOAT_TRUNC",
            PcodeOp::FLOAT_CEIL => "FLOAT_CEIL",
            PcodeOp::FLOAT_FLOOR => "FLOAT_FLOOR",
            PcodeOp::FLOAT_ROUND => "FLOAT_ROUND",
            PcodeOp::MULTIEQUAL => "MULTIEQUAL",
            PcodeOp::INDIRECT => "INDIRECT",
            PcodeOp::PIECE => "PIECE",
            PcodeOp::SUBPIECE => "SUBPIECE",
            PcodeOp::CAST => "CAST",
            PcodeOp::PTRADD => "PTRADD",
            PcodeOp::PTRSUB => "PTRSUB",
            PcodeOp::SEGMENTOP => "SEGMENTOP",
            PcodeOp::CPOOLREF => "CPOOLREF",
            PcodeOp::NEW => "NEW",
            PcodeOp::INSERT => "INSERT",
            PcodeOp::ZPULL => "ZPULL",
            PcodeOp::POPCOUNT => "POPCOUNT",
            PcodeOp::LZCOUNT => "LZCOUNT",
            PcodeOp::SPULL => "SPULL",
            PcodeOp::PCODE_MAX => "PCODE_MAX",
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
