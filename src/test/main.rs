use sum_error::*;

#[derive(SumError)]
enum A {
    A(std::io::Error),
}

// #[derive(SumError)]
// enum B {
//     A(std::io::Error),
//     B(std::io::Error),
// }
//
// #[derive(SumError)]
// struct C { x: i32 }
//
// #[derive(SumError)]
// enum D {
//     A {a: std::io::Error},
// }
//
// #[derive(SumError)]
// enum E {
//     A(std::io::Error, std::io::Error),
// }

fn main() {

}
