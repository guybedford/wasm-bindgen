export class TestValue {
  constructor(value) {
    this._value = value;
  }

  get value() {
    return this._value;
  }

  transform(suffix) {
    return new TestValue(this._value + suffix);
  }
}

export class TestResult {
  constructor(success, data) {
    this._success = success;
    this._data = data;
  }

  get success() {
    return this._success;
  }

  get data() {
    return this._data;
  }
}

export function createTestValuePromise(value) {
  return Promise.resolve(new TestValue(value));
}

export function createTestResultPromise(success, value) {
  return Promise.resolve(new TestResult(success, new TestValue(value)));
}

export function processTestValuePromise(promise) {
  return promise.then((val) => val.transform("_processed"));
}

export function chainTestValuePromises(promise1, promise2) {
  return Promise.all([promise1, promise2]).then(
    ([v1, v2]) => new TestValue(v1.value + "+" + v2.value)
  );
}

export async function checkTestValuePromise(promise) {
  const val = await promise;
  if (!(val instanceof TestValue)) {
    throw new Error("Expected TestValue instance");
  }
}
