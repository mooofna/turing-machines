// Replaces every x by X, with three mistakes.
machine "typo"
Q = {q0, q1}
F = {q1}

δ(q0, x) = (q0, X, R)
δ(q0, x) = (q1, X, L)
δ(q0, ␣) = (qq1, ␣, L)
δ(q1, X) = (q1, X, Q)
