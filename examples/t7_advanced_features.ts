function main() {
  console.log("=== RUNNING ADVANCED FEATURES TEST ===");

  // 1. Prototype property existence check using "in"
  const user = { name: "John", age: 25 };
  console.log("Checking property existence using 'in':");
  console.log("name" in user);
  console.log("role" in user);

  // 2. Optional chaining and nullish coalescing
  const profile = {
    personal: {
      firstName: "Jane",
      lastName: "Doe"
    },
    preferences: null
  };

  const middleName = profile?.personal?.middleName ?? "No Middle Name";
  const theme = profile?.preferences?.theme ?? "light-mode";

  console.log("Optional chaining + Nullish coalescing results:");
  console.log(middleName);
  console.log(theme);

  // 3. Dynamic indexing for keys
  const data = { score1: 95, score2: 88, score3: 100 };
  const key1 = "score1";
  const key2 = "score3";

  console.log("Dynamic index get results:");
  console.log(data[key1]);
  console.log(data[key2]);

  console.log("Dynamic index set results:");
  data[key1] = 98;
  console.log(data[key1]);

  console.log("=== ADVANCED FEATURES TEST COMPLETED SUCCESSFULLY ===");
}
main();
