export class TestItem {
  constructor(id, name) {
    this._id = id;
    this._name = name;
  }

  get id() {
    return this._id;
  }

  get name() {
    return this._name;
  }

  with_prefix(prefix) {
    return new TestItem(this._id, prefix + this._name);
  }
}

export function createTestItemArray() {
  return [
    new TestItem(1, "first"),
    new TestItem(2, "second"),
    new TestItem(3, "third"),
  ];
}

export function processTestItemArray(arr) {
  return arr.reduce((sum, item) => sum + item.id, 0);
}

export function checkArrayType(arr) {
  return Array.isArray(arr) && arr.every((item) => item instanceof TestItem);
}
