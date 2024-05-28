use std::io::Result;

#[test]
pub fn ls() -> Result<()> {
	Ok(println!("{:?}", super::ls("./")?.collect::<Vec<_>>()))
}
#[test]
pub fn cat() -> Result<()> {
	Ok(println!("{:?}", super::cat("./src/tests.rs")?))
}
