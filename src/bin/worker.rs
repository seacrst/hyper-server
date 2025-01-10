use collect_args::Args;

fn main() {

  let args = Args::collect();

  
  let (_, ports) = args.input("ports");
  
  print!("{}", ports.unwrap_or(String::from("No ports")));
}