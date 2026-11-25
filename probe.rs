use std::fmt::Write as _;
use std::path::Path;
use xcl::gpu::{dxil::{kernel, validate}, rdna2, recomp};

fn inspect(path: &Path, label: &str, program: &[u8], expected_width: u32) -> Result<(), String> {
    validate::validate(program)?;
    let assembly = validate::disassemble(program)?;
    std::fs::write(path.join(format!("{label}.ll")), &assembly).map_err(|e| e.to_string())?;
    let operations: Vec<_> = assembly.lines()
        .filter(|s| s.contains("call ") && s.contains("@dx.op.atomicBinOp.i") && s.contains("i32 7,"))
        .collect();
    let width = match operations.as_slice() {
        [op] if op.contains("@dx.op.atomicBinOp.i32(") => 32,
        [op] if op.contains("@dx.op.atomicBinOp.i64(") => 64,
        _ => return Err(format!("{label}: unexpected UMAX count {}", operations.len())),
    };
    println!("{label} compiled validated umax=i{width} expected=i{expected_width} match={}", width == expected_width);
    Ok(())
}

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("output path required")?;
    let path = Path::new(&path);
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    for (mask, glc) in [(1, 0), (3, 0), (3, 1)] {
        let fields = [("VDATA", 22), ("VADDR", 10), ("SRSRC", 4), ("DMASK", mask), ("DIM", 1), ("UNORM", 1), ("GLC", glc)];
        let mut code = rdna2::encode("IMAGE_ATOMIC_UMAX", "ENC_MIMG", &fields, None)?;
        code.extend(rdna2::encode("S_ENDPGM", "ENC_SOPP", &[], None)?);
        let mut hex = String::new();
        for b in &code { write!(&mut hex, "{b:02x}").unwrap(); }
        println!("dmask={mask} glc={glc} code={hex}");
        let expected_width = if mask == 3 { 64 } else { 32 };
        let hlsl = recomp::translate(&code, &recomp::Options::default())?;
        if !hlsl.complete() { return Err(format!("HLSL refused {:?}", hlsl.unsupported_why)); }
        let label = format!("hlsl_mask{mask}_glc{glc}");
        std::fs::write(path.join(format!("{label}.hlsl")), &hlsl.hlsl).map_err(|e| e.to_string())?;
        let compiled = recomp::dxc::compile(&hlsl.hlsl, "main", "cs_6_6", &[])?;
        inspect(path, &label, &compiled, expected_width)?;
        let direct = kernel::translate(&code, kernel::KernelOptions { wave64: true, split: false, mem_windows: 1, sampler_base: 0 });
        match direct {
            Ok(dxil) => inspect(path, &format!("direct_mask{mask}_glc{glc}"), &dxil, expected_width)?,
            Err(message) => println!("direct_mask{mask}_glc{glc}: refused {message}"),
        }
    }
    Ok(())
}
