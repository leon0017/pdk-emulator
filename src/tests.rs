#[test]
fn ram_test() {
    let mut cpu = crate::default_cpu();
    assert_eq!(cpu.ram_get(0), 0x00);
    cpu.ram_write(0, 0xAB);
    assert_eq!(cpu.ram_get(0), 0xAB);
}

#[test]
fn stack_test8() {
    let mut cpu = crate::default_cpu();
    cpu.stack_push8(0xAB);
    cpu.stack_push8(0xCD);
    assert_eq!(cpu.stack_pop8(), 0xCD);
    assert_eq!(cpu.stack_pop8(), 0xAB);
    assert_eq!(cpu.stack_pointer(), 0);
}

#[test]
fn stack_test16() {
    let mut cpu = crate::default_cpu();
    cpu.stack_push16(0xABCD);
    cpu.stack_push16(0x1234);
    assert_eq!(cpu.stack_pop16(), 0x1234);
    assert_eq!(cpu.stack_pop16(), 0xABCD);
    assert_eq!(cpu.stack_pointer(), 0);
}
