# Advent-Of-Code-2023
2023 Advent of Code solutions

# Day 8
I approached part 2 expecting another Chinese remainder theorem problem.
After coprimes didn't work out,
and I spent a while on the problem,
I looked at discussion of the problem and did it as GCD.
That's really more of a coincidence of the input then a statement of the problem, though.
So I added checks for the GCD approach,
checks for a CRT approach (using extended GCD),
and a naive approach if conditions fail for both the above.
That should work for any input as long as it doesn't get trapped in a loop that prevents it from reaching Z...
