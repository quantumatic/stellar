pattern (Even | Odd) (n: uint32) {
    if n % 2 == 0 {
        Even
    } else {
        Odd
    }
}

match f {
    Even => println("hello"), 
    Odd => println("hello") 
}

