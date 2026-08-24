use rand::distributions::uniform::SampleUniform;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;

/// Класс символов
pub enum CharClass {
  Numeric,
  Letter,
  Multi,
  Special,
}

/// Вспомогательная функция выбора случайного символа из строки
pub fn randchar(s: &str) -> char {
  let v: Vec<char> = s.chars().collect();
  let idx = thread_rng().gen_range(0..v.len());
  v[idx]
}

/// Функция генерации строки
pub fn randstr(class: CharClass, length: i32) -> String {
  let chars = match class {
    CharClass::Numeric => "0123456789",
    CharClass::Letter => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    CharClass::Multi => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    CharClass::Special => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_-+=",
  };

  let mut result = String::new();

  for _ in 0..length {
    result.push(randchar(chars));
  }

  result
}

/// Функция генерации случайного числа в заданном диапозоне
pub fn randnum<N>(min: N, max: N) -> N
where
  N: SampleUniform + PartialOrd,
{
  thread_rng().gen_range(min..=max)
}

/// Функция выбора случайнного элемента из массива
pub fn randelem<T>(arr: &[T]) -> &T
where
  T: Sized,
{
  arr.choose(&mut thread_rng()).unwrap_or(&arr[0])
}

/// Функция генерации случайного шанса
pub fn randchance(percent: f64) -> bool {
  thread_rng().gen_bool(percent)
}
