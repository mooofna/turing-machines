// An acceptor for {aⁿbⁿcⁿ | n ∈ ℕ}, a language that is not context-free.
//
// Each round marks one a as X, one b as Y and one c as Z. When no a is
// left, only Y and Z may remain. Any other symbol leaves the machine
// without a transition, so it halts outside F and rejects.
machine "aⁿbⁿcⁿ"
kind acceptor
input "aabbcc"

Q = {q0, q1, q2, q3, q4, qacc}
Σ = {a, b, c}
Γ = {a, b, c, X, Y, Z, ␣}
start = q0
F = {qacc}

δ(q0, a) = (q1, X, R)    // mark an a
δ(q0, Y) = (q4, Y, R)    // no a left: check the rest
δ(q0, ␣) = (qacc, ␣, R)  // the empty word

δ(q1, {a, Y}) = (q1, *, R)
δ(q1, b) = (q2, Y, R)    // mark a b

δ(q2, {b, Z}) = (q2, *, R)
δ(q2, c) = (q3, Z, L)    // mark a c

δ(q3, {a, b, Y, Z}) = (q3, *, L)
δ(q3, X) = (q0, X, R)    // back to the first unmarked a

δ(q4, {Y, Z}) = (q4, *, R)
δ(q4, ␣) = (qacc, ␣, L)
