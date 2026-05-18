function assertEqual(name: string, actual: any, expected: any) {
  if (actual === expected) {
    console.log(`Assert [${name}]: expected ${expected}, got ${actual} → PASS`);
  } else {
    console.error(`Assert [${name}]: expected ${expected}, got ${actual} → FAIL`);
    throw new Error(`Assertion failed: ${name}`);
  }
}

// 1. Tagged Template Literals
function sql(strings: string[], ...values: any[]) {
  let result = strings[0];
  for (let i = 0; i < values.length; i++) {
    result += values[i] + strings[i + 1];
  }
  return result;
}

// 2. Destructuring with Rest and Defaults
function testDestructuring() {
  const arr = [10, 20, 30, 40, 50];
  const [first, second, ...restArr] = arr;
  assertEqual("Array destructuring first", first, 10.0);
  assertEqual("Array destructuring second", second, 20.0);
  assertEqual("Array destructuring rest[0]", restArr[0], 30.0);
  assertEqual("Array destructuring rest length", restArr.length, 3.0);

  const obj = { x: 100, y: 200, z: { nested: 300 } };
  // Note: we use any to bypass strict type checking for the rest/default extraction for demonstration
  const { x, z: { nested }, w = 400, ...restObj } = obj as any;
  assertEqual("Object destructuring x", x, 100.0);
  assertEqual("Object destructuring nested", nested, 300.0);
  assertEqual("Object destructuring default w", w, 400.0);
  assertEqual("Object destructuring restObj.y", restObj.y, 200.0);
  assertEqual("Object destructuring restObj.x", restObj.x, null); // TSDroid dynamic lookup returns null for missing props
}

// 3. TS Advanced Types (Erasure Validation)
type MyComplexType<T> = T extends string ? "A" : "B";
interface Node<T> {
  value: T;
  next?: Node<T>;
}
type ReadonlyNode<T> = Readonly<Node<T>>;

function typeSystemTest(input: MyComplexType<number>): ReadonlyNode<number> {
  return { value: 42.0 } as any;
}

// 4. Classes with static blocks and private fields
class AdvancedClass {
  static #privateStatic = "secret";
  #privateInstance = 10;

  static {
    AdvancedClass.#privateStatic = "initialized secret";
  }

  getSecret() {
    return AdvancedClass.#privateStatic + " - " + this.#privateInstance;
  }
}

async function main() {
  console.log("=== RUNNING ULTIMATE COMPILER TEST SUITE ===");

  // Tagged Templates
  const table = "users";
  const id = 42;
  const query = sql`SELECT * FROM ${table} WHERE id = ${id}`;
  assertEqual("Tagged Template String", query, "SELECT * FROM users WHERE id = 42");

  // Destructuring
  testDestructuring();

  // Type System Erasure
  const node = typeSystemTest("B" as any);
  assertEqual("Type Erasure Return", node.value, 42.0);

  // Advanced Classes
  const instance = new AdvancedClass();
  assertEqual("Advanced Class Output", instance.getSecret(), "initialized secret - 10");

  console.log("=== ALL ULTIMATE TESTS COMPLETED SUCCESSFULLY ===");
}

main();
