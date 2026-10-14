// An acceptor for the palindromes over {a, b}.
//
// The machine erases the first symbol, remembers it in its state and
// compares it with the last symbol, which it erases too. A mismatch
// leaves the machine in qa' or qb' without a transition, so it halts
// outside F and rejects. When nothing is left, the word was a palindrome.
machine "Palindromes"
kind acceptor
input "abba"

Q = {q0, qa, qb, qa', qb', qret, qacc}
Σ = {a, b}
Γ = {a, b, ␣}
start = q0
F = {qacc}

δ(q0, a) = (qa, ␣, R)    // take the first symbol
δ(q0, b) = (qb, ␣, R)
δ(q0, ␣) = (qacc, ␣, R)  // nothing left: accept

// Run to the last symbol.
δ(qa, {a, b}) = (qa, *, R)
δ(qa, ␣) = (qa', ␣, L)
δ(qb, {a, b}) = (qb, *, R)
δ(qb, ␣) = (qb', ␣, L)

// It must match the first one.
δ(qa', a) = (qret, ␣, L)
δ(qa', ␣) = (qacc, ␣, R)  // the a was the middle symbol
δ(qb', b) = (qret, ␣, L)
δ(qb', ␣) = (qacc, ␣, R)

// Back to the front for the next round.
δ(qret, {a, b}) = (qret, *, L)
δ(qret, ␣) = (q0, ␣, R)
