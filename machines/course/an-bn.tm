// An enumerator for {aⁿbⁿ | n ∈ ℕ}.
//
// TheoLog 2024, exercise sheet 1, task 2b. In the output state q0 the
// tape holds aⁿbⁿ and the head stands on its first symbol. Each round
// puts one a in front of the word and one b behind it, then returns to
// the front. The tape is infinite in both directions, so there is always
// room on the left.
machine "aⁿbⁿ"
kind enumerator

Q = {q0, q1, q2, q3}
Γ = {a, b, ␣}
start = q0
F = {q0}

δ(q0, ␣) = (q2, a, R)  // n = 0: the first a goes on the empty tape
δ(q0, a) = (q1, a, L)  // n > 0: step in front of the word

δ(q1, ␣) = (q2, a, R)  // one more a at the front

// Run to the end of the word and append one b.
δ(q2, {a, b}) = (q2, *, R)
δ(q2, ␣) = (q3, b, L)

// Back to the front, then output the tape.
δ(q3, {a, b}) = (q3, *, L)
δ(q3, ␣) = (q0, ␣, R)
