mod tax;
pub fn receipt(amount: u64) -> u64 {
    tax::add_tax(amount)
}
