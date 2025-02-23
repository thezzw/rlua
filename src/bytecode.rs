#[derive(Debug)]
pub enum Bytecode {
    GetGlobal(u8, u8),
    LoadConst(u8, u8),
    Call(u8, u8)
}