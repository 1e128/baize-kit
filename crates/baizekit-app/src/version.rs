use once_cell::sync::OnceCell;

pub static GLOBAL_VERSION_PRINTER: OnceCell<fn()> = OnceCell::new();

/// 生成版本信息（根据feature决定是否包含git信息）
#[cfg(feature = "build-version")]
pub fn generate_version() -> Result<(), Box<dyn std::error::Error>> {
    use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder, SysinfoBuilder};

    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gitcl = GixBuilder::all_git()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;
    Ok(())
}
