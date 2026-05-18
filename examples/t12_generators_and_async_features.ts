function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

// === 1. TS Decorators (cleanly parsed and erased) ===
function logged(target: any, key: string, descriptor: any) {
  return descriptor;
}

class UserProfile {
  name: string;
  role: string;

  constructor(name: string, role: string) {
    this.name = name;
    this.role = role;
  }

  @logged
  getDetails() {
    return this.name + " (" + this.role + ")";
  }
}

// === 2. Advanced TS Type System Constructs (type-checked and erased) ===
type Point = { x: number; y: number };
type Point3D = Point & { z: number }; // Intersection Type

type ReadonlyPoint = { readonly [P in keyof Point]: Point[P] }; // Mapped Type & keyof
type IsString<T> = T extends string ? true : false; // Conditional Type
type InferElement<T> = T extends (infer U)[] ? U : T; // infer keyword

// === 3. Generators and yield / yield* ===
function* fibonacci(limit: number) {
  let a = 0;
  let b = 1;
  let count = 0;
  while (count < limit) {
    yield a;
    let next = a + b;
    a = b;
    b = next;
    count = count + 1;
  }
}

// Sub-generator for yield* delegation
function* innerGen() {
  yield 100;
  yield 200;
}

function* outerGen() {
  yield* innerGen(); // yield* delegation
  yield 300;
}

// === 4. Async Generators and for await...of loops ===
async function* asyncCountdown(start: number) {
  let current = start;
  while (current > 0) {
    yield current;
    current = current - 1;
  }
}

async function runAsyncLoop() {
  let sum = 0;
  const gen = asyncCountdown(3);
  for await (const x of gen) {
    sum = sum + x;
  }
  return sum;
}

async function runTests() {
  console.log("=== RUNNING GENERATORS & ASYNC COMPILER TEST SUITE ===");

  // Test Decorator-erased class details
  const user = new UserProfile("Alice", "Admin");
  assertEqual(user.getDetails(), "Alice (Admin)", "Decorator method invocation");

  // Test standard generator
  console.log("\n--- Testing standard generator (Fibonacci) ---");
  const fib = fibonacci(5);
  assertEqual(fib.next().value, 0, "Fibonacci term 1");
  assertEqual(fib.next().value, 1, "Fibonacci term 2");
  assertEqual(fib.next().value, 1, "Fibonacci term 3");
  assertEqual(fib.next().value, 2, "Fibonacci term 4");
  assertEqual(fib.next().value, 3, "Fibonacci term 5");
  assertEqual(fib.next().done, true, "Fibonacci finished");

  // Test yield* delegation generator
  console.log("\n--- Testing yield* delegation generator ---");
  const out = outerGen();
  assertEqual(out.next().value, 100, "yield* term 1");
  assertEqual(out.next().value, 200, "yield* term 2");
  assertEqual(out.next().value, 300, "outer term 3");
  assertEqual(out.next().done, true, "outer finished");

  // Test async generators and for await...of loops
  console.log("\n--- Testing async generators & for await...of loops ---");
  const asyncResultPromise = runAsyncLoop();
  
  // Await the CompletableFuture on the JVM
  const finalSum = await asyncResultPromise;
  assertEqual(finalSum, 6, "Sum of async countdown 3 + 2 + 1");

  console.log("\n=== ALL RECENTLY ACHIEVED FEATURES COMPLETED SUCCESSFULLY ===");
}

async function main() {
  const promise = runTests();
  await promise;
}

main();
