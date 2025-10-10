use rand::Rng;

pub(crate) fn credits() {
  let mut rng = rand::rng();
  match rng.random_range(1..=10)  {
    1 => println!("💖 Поля"),
    _ => println!("You are the best!"),
  };
}