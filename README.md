# bad-lang-2

## Basic Example

```rs
fn fib(n) {
  let a = 0
  let b = 1
  let temp = 0

  if (#eq(n, 0)) {
    return a
  }

  let i = 2
  loop {
    if (#gt(i, n)) {
      break
    }

    temp = a
    a = b
    b += temp

    i++
  }

  return b
}

io#println(fib(10))

fn factorial(n) {
  if (#eq(n, 0)) {
    return 1
  }

  if (#lt(n, 0)) {
    io#println("Error: Factorial not defined for negative numbers")
    return -1
  }

  let result = 1
  let i = 1

  loop {
    if (#gt(i, n)) {
      break
    }

    result *= i
    i++
  }

  return result
}

io#println(factorial(10))
```
