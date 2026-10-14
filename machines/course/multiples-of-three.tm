// An enumerator for {x^(3n) | n ∈ ℕ}, the multiples of three in unary.
//
// TheoLog 2024, exercise sheet 1, task 2a. The output state q0 is also
// the start state, so the empty word (n = 0) is output first; after that,
// every third x brings the machine back to q0, which outputs the tape.
machine "Multiples of three"
kind enumerator
output unary x

Q = {q0, q1, q2}
Γ = {x, ␣}
start = q0
F = {q0}

δ(q0, ␣) = (q1, x, R)
δ(q1, ␣) = (q2, x, R)
δ(q2, ␣) = (q0, x, R)  // three more x: output the tape
