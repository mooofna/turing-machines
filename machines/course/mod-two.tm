// A_mod2: x^n becomes x^(n mod 2).
//
// TheoLog 2024, exercise sheet 3, task E. The machine runs to the end of
// the input and erases it from the right, keeping the parity of the
// number of erased x in its state. At the left end it writes one x if
// that number is odd.
machine "A_mod2"
input "xxxxx"
output unary x

Q = {q0, even, odd, qf}
Σ = {x}
Γ = {x, ␣}
start = q0
F = {qf}

δ(q0, x) = (q0, x, R)    // run to the end of the input
δ(q0, ␣) = (even, ␣, L)

δ(even, x) = (odd, ␣, L)
δ(even, ␣) = (qf, ␣, R)  // n is even: the tape stays empty

δ(odd, x) = (even, ␣, L)
δ(odd, ␣) = (qf, x, R)   // n is odd: one x remains
