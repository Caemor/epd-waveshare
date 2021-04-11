#[rustfmt::skip]
pub(crate) const LUT_VCOM_DC: [u8; 44] = [
0x00, 0x00,
0x00, 0x1A, 0x1A, 0x00, 0x00, 0x01,
0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x00, 0x0E, 0x01, 0x0E, 0x01, 0x10,
0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x00, 0x04, 0x10, 0x00, 0x00, 0x05,
0x00, 0x03, 0x0E, 0x00, 0x00, 0x0A,
0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

#[rustfmt::skip]
pub(crate) const LUT_WW: [u8; 42] =[
0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01,
0x40, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x84, 0x0E, 0x01, 0x0E, 0x01, 0x10,
0x80, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x00, 0x04, 0x10, 0x00, 0x00, 0x05,
0x00, 0x03, 0x0E, 0x00, 0x00, 0x0A,
0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

#[rustfmt::skip]
pub(crate) const LUT_BW: [u8; 42] =[
0xA0, 0x1A, 0x1A, 0x00, 0x00, 0x01,
0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x84, 0x0E, 0x01, 0x0E, 0x01, 0x10,
0x90, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0xB0, 0x04, 0x10, 0x00, 0x00, 0x05,
0xB0, 0x03, 0x0E, 0x00, 0x00, 0x0A,
0xC0, 0x23, 0x00, 0x00, 0x00, 0x01,
];

#[rustfmt::skip]
pub(crate) const LUT_BB: [u8; 42] =[
0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01,
0x40, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x84, 0x0E, 0x01, 0x0E, 0x01, 0x10,
0x80, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x00, 0x04, 0x10, 0x00, 0x00, 0x05,
0x00, 0x03, 0x0E, 0x00, 0x00, 0x0A,
0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

#[rustfmt::skip]
pub(crate) const LUT_WB: [u8; 42] =[
0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01,
0x20, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x84, 0x0E, 0x01, 0x0E, 0x01, 0x10,
0x10, 0x0A, 0x0A, 0x00, 0x00, 0x08,
0x00, 0x04, 0x10, 0x00, 0x00, 0x05,
0x00, 0x03, 0x0E, 0x00, 0x00, 0x0A,
0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];