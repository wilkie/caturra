use std::io::Read;
fn find_main(s: &[jvmjs_compiler::SourceFile]) -> Option<String> {
    for f in s { if let Some(i)=f.text.find("static void main"){ if let Some(c)=f.text[..i].rfind("class "){ let n:String=f.text[c+6..].chars().take_while(|c|c.is_alphanumeric()||*c=='_').collect(); if !n.is_empty(){return Some(n);}}}} None }
fn main() {
    let mut buf=String::new(); std::fs::File::open(std::env::args().nth(1).unwrap()).unwrap().read_to_string(&mut buf).unwrap();
    let mut files=Vec::new(); let mut grid=String::new(); let mut data:Vec<(String,String)>=Vec::new();
    for p in buf.split('\x1e'){
        if let Some(r)=p.strip_prefix("F:"){ if let Some((n,t))=r.split_once('\x1f'){ files.push(jvmjs_compiler::SourceFile{path:n.into(),text:t.into()});}}
        else if let Some(r)=p.strip_prefix("D:"){ if let Some((n,t))=r.split_once('\x1f'){ data.push((n.into(),t.into())); }}
        else if let Some(g)=p.strip_prefix("G:"){ grid=g.into(); }
    }
    let c=jvmjs_compiler::compile(&files);
    if !c.success(){ let e=c.diagnostics.iter().find(|d|matches!(d.severity,jvmjs_compiler::diagnostics::Severity::Error)); println!("CE: {}",e.map(|d|d.message.clone()).unwrap_or_default()); return; }
    let entry=c.validation_entry.clone().unwrap_or_else(||find_main(&files).unwrap_or_default());
    let mut vfs=jvmjs_vm::VirtualFileSystem::new(); if !grid.is_empty(){ vfs.write_file("grid.txt",grid.into_bytes()).unwrap(); }
    for (n,t) in &data { vfs.write_file(n, t.clone().into_bytes()).unwrap(); }
    let input: Vec<String> = (0..200).map(|_| String::from("7")).collect();
    let mut con=jvmjs_vm::BufferedConsole::with_input(input);
    let opts = jvmjs_vm::VmOptions { max_instructions: 20_000_000, ..jvmjs_vm::VmOptions::default() };
    let mut vm=jvmjs_vm::Vm::new(opts,&mut vfs,&mut con);
    for cl in c.classes{ if let Err(e)=vm.load_class(cl.class_file){println!("LOAD: {e:?}");return;} }
    match vm.run_main(&entry,&[]){
        Ok(_)=>{ let vt=con.stdout_text().lines().filter(|l|l.starts_with("__VTEST")).count(); let n=vfs.read_file("neighborhood.jsonl").map(|b|String::from_utf8_lossy(b).lines().count()).unwrap_or(0); println!("OK {n} VT{vt}"); }
        Err(jvmjs_vm::VmError::InstructionBudgetExceeded)=>println!("BUDGET"),
        Err(e)=>println!("RUN: {e:?}"),
    }
}
