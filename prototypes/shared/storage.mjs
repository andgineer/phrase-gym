const NAME = "phrase-gym-p1";
let opening;
function database() {
  opening ||= new Promise((resolve, reject) => {
    const request = indexedDB.open(NAME, 1);
    request.onupgradeneeded = () => {
      for (const name of [
        "settings",
        "sessions",
        "takes",
        "events",
        "references",
      ]) {
        request.result.createObjectStore(name, {
          keyPath: name === "sessions" ? "language" : "id",
        });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => {
      opening = null;
      reject(request.error);
    };
    request.onblocked = () =>
      reject(
        new Error("Close the other prototype tabs to open local storage."),
      );
  });
  return opening;
}

async function transaction(store, mode, operation) {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, mode);
    const request = operation(tx.objectStore(store));
    let result;
    request.onsuccess = () => {
      result = request.result;
    };
    tx.oncomplete = () => resolve(result);
    tx.onabort = tx.onerror = () =>
      reject(tx.error || request.error || new Error("Local storage failed."));
  });
}
export const get = (store, id) =>
  transaction(store, "readonly", (s) => s.get(id));
export const all = (store) => transaction(store, "readonly", (s) => s.getAll());
export const put = (store, value) =>
  transaction(store, "readwrite", (s) => s.put(value));
// Read and change one record inside the same transaction, including across tabs.
export async function update(store, id, change) {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, "readwrite");
    const records = tx.objectStore(store);
    const request = records.get(id);
    let value;
    request.onsuccess = () => {
      try {
        value = change(request.result);
        records.put(value);
      } catch (error) {
        tx.abort();
        reject(error);
      }
    };
    tx.oncomplete = () => resolve(value);
    tx.onabort = tx.onerror = () =>
      reject(tx.error || request.error || new Error("Local storage failed."));
  });
}
export const remove = (store, id) =>
  transaction(store, "readwrite", (s) => s.delete(id));
export const clear = (store) =>
  transaction(store, "readwrite", (s) => s.clear());
export const event = (type, metadata = {}) =>
  put("events", { id: crypto.randomUUID(), type, at: Date.now(), ...metadata });

// A grade/correction, its receipt and its checkpoint must all commit or all fail.
export async function commitSessionEvent(session, event, initialHistory) {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(["settings", "events", "sessions"], "readwrite");
    const settings = tx.objectStore("settings");
    const request = settings.get("history");
    let history;
    request.onsuccess = () => {
      try {
        history = request.result || structuredClone(initialHistory);
        if (event.type === "result" && event.scheduleUpdated) {
          history.reviewed[event.targetId] = {
            eventId: event.id,
            result: event.result,
            at: event.at,
          };
        }
        if (
          event.type === "undo" &&
          history.reviewed[event.targetId]?.eventId === event.eventId
        ) {
          delete history.reviewed[event.targetId];
        }
        settings.put(history);
        tx.objectStore("events").put(event);
        tx.objectStore("sessions").put(session);
      } catch (error) {
        tx.abort();
        reject(error);
      }
    };
    tx.oncomplete = () => resolve(history);
    tx.onabort = tx.onerror = () =>
      reject(
        tx.error || request.error || new Error("Local result storage failed."),
      );
  });
}
