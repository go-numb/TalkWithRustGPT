/**
 * Serializes requests while retaining one intentional follow-up request.
 */
export class RequestQueue<T> {
  private active = false;
  private next: T | null = null;

  get isRunning(): boolean {
    return this.active;
  }

  get pending(): T | null {
    return this.next;
  }

  reserve(item: T): boolean {
    if (!this.active || this.next !== null) {
      return false;
    }

    this.next = item;
    return true;
  }

  cancel(): T | null {
    const item = this.next;
    this.next = null;
    return item;
  }

  async run(first: T, send: (item: T) => Promise<void>): Promise<void> {
    if (this.active) {
      throw new Error('A request is already running.');
    }

    this.active = true;
    try {
      let item: T | null = first;
      while (item !== null) {
        await send(item);
        item = this.cancel();
      }
    } catch (error) {
      this.cancel();
      throw error;
    } finally {
      this.active = false;
    }
  }
}
