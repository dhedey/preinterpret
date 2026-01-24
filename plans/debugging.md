The following are specific issues you might encounter, and what they might mean.

## SIGBUS

It appears that a stack overflow of a proc macro (at least on an M2 Macbook in Oct 2025) gives a SIGBUS error. Look for accidental recursion in the methods to identify the likely cause.