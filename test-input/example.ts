interface User {
    name: string;
    age: number;
    email?: string;
}

class UserManager {
    private users: User[] = [];

    constructor() {
        console.log("UserManager initialized");
    }

    addUser(user: User): void {
        this.users.push(user);
        console.log(`Added user: ${user.name}`);
    }

    getUsers(): User[] {
        return this.users;
    }

    findUserByName(name: string): User | undefined {
        return this.users.find(user => user.name === name);
    }
}

const manager = new UserManager();
manager.addUser({
    name: "John Doe",
    age: 30,
    email: "john@example.com"
});

export { UserManager, User };