# Test Input Files - TypeScript Compressor Testing Suite

Sample TypeScript files designed to test and demonstrate the capabilities of the `ts-compressor` Rust tool. These files serve as both functional tests and interview demonstration materials.

## 🎯 Purpose

### Testing Scenarios
- **Basic TypeScript Features**: Type annotations, interfaces, classes
- **Advanced Language Features**: Generics, decorators, async/await
- **Error Handling**: Malformed syntax, type errors, edge cases
- **Performance Benchmarking**: Large files, complex dependency graphs

### Interview Demonstration
- **Tool Validation**: Show the compressor working on real TypeScript code
- **Before/After Comparison**: Demonstrate minification and optimization results
- **Error Handling**: Show graceful handling of problematic input files
- **Performance Analysis**: Benchmark compilation speed and output size

## 📁 File Structure

```
test-input/
├── README.md           # This documentation
├── example.ts          # Basic TypeScript demonstration file
├── complex.tsx         # Advanced features with JSX
├── error-cases/        # Files with intentional errors
│   ├── syntax-error.ts
│   ├── type-error.ts
│   └── missing-import.ts
├── performance/        # Large files for benchmarking
│   ├── large-class.ts
│   └── many-imports.ts
└── real-world/         # Realistic code examples
    ├── api-client.ts
    ├── data-models.ts
    └── utility-functions.ts
```

## 🧪 Test Cases Covered

### Basic TypeScript Features (`example.ts`)
```typescript
// Type annotations and interfaces
interface User {
    id: number;
    name: string;
    email?: string;
}

// Classes with inheritance
class UserService {
    private users: User[] = [];
    
    async createUser(userData: Omit<User, 'id'>): Promise<User> {
        const user: User = {
            id: Date.now(),
            ...userData
        };
        this.users.push(user);
        return user;
    }
}

// Generic functions
function mapArray<T, U>(arr: T[], fn: (item: T) => U): U[] {
    return arr.map(fn);
}
```

### Advanced Features Testing
- **Generics**: Complex type parameters and constraints
- **Decorators**: Class and method decorators
- **Async/Await**: Promise-based asynchronous code
- **Union Types**: Complex type unions and intersections
- **Modules**: Import/export statements and module resolution

### JSX and React Components (`complex.tsx`)
```typescript
import React, { useState, useEffect } from 'react';

interface Props {
    initialCount?: number;
    onCountChange?: (count: number) => void;
}

const Counter: React.FC<Props> = ({ initialCount = 0, onCountChange }) => {
    const [count, setCount] = useState<number>(initialCount);
    
    useEffect(() => {
        onCountChange?.(count);
    }, [count, onCountChange]);
    
    return (
        <div className="counter">
            <button onClick={() => setCount(c => c - 1)}>-</button>
            <span>{count}</span>
            <button onClick={() => setCount(c => c + 1)}>+</button>
        </div>
    );
};
```

## 🔧 Usage Examples

### Basic Compilation Testing
```bash
# Test basic TypeScript compilation
cd ts-compressor
cargo run -- ../test-input/example.ts

# Test JSX compilation
cargo run -- ../test-input/complex.tsx

# Test directory processing
cargo run -- --recursive ../test-input/
```

### Performance Benchmarking
```bash
# Benchmark compilation speed
time cargo run --release -- ../test-input/performance/large-class.ts

# Compare output sizes
ls -la test-input/example.ts
cargo run -- test-input/example.ts > output.js
ls -la output.js

# Test minification effectiveness
cargo run -- --minify test-input/example.ts > minified.js
wc -c test-input/example.ts output.js minified.js
```

### Error Handling Validation
```bash
# Test graceful error handling
cargo run -- test-input/error-cases/syntax-error.ts
cargo run -- test-input/error-cases/type-error.ts
cargo run -- test-input/error-cases/missing-import.ts
```

## 📊 Expected Results

### Compilation Success Metrics
- **Basic TypeScript**: Should compile without errors, types stripped
- **JSX Components**: Should transform JSX to React.createElement calls
- **Complex Features**: Should handle generics, decorators, async code
- **Error Cases**: Should provide helpful error messages, not crash

### Performance Expectations
- **Small Files (<10KB)**: Sub-millisecond compilation
- **Medium Files (10-100KB)**: Under 10ms compilation
- **Large Files (>100KB)**: Linear scaling with file size
- **Minification**: 30-50% size reduction typical

### Output Quality Checks
```javascript
// Before (TypeScript)
interface User {
    id: number;
    name: string;
}

const createUser = (name: string): User => ({
    id: Math.random(),
    name
});

// After (Minified JavaScript)
const createUser=e=>({id:Math.random(),name:e});
```

## 🎓 Interview Discussion Points

### Tool Validation Questions
**Q: How do you verify the compressor produces correct output?**
- Compare with official TypeScript compiler output
- Run generated JavaScript in Node.js/browser
- Test with existing test suites
- Validate source map accuracy

**Q: What edge cases does your test suite cover?**
- Malformed syntax and recovery strategies
- Large files and memory usage
- Complex type systems and generics
- Module resolution and dependency handling

### Performance Analysis
**Q: How do you benchmark compilation performance?**
- Time measurement with various file sizes
- Memory usage profiling with valgrind
- Comparison with tsc and other tools
- Scalability testing with large codebases

### Quality Assurance
**Q: How do you ensure output correctness?**
- Automated testing with known good inputs
- Runtime validation of generated JavaScript
- Comparison testing against reference implementations
- Regression testing for bug fixes

## 🚀 Extending the Test Suite

### Adding New Test Cases
1. **Create Test File**: Add new `.ts` or `.tsx` file with specific features
2. **Document Expected Behavior**: Add comments explaining what should happen
3. **Add to CI Pipeline**: Include in automated testing
4. **Benchmark if Needed**: Add performance expectations

### Test Categories to Consider
- **Language Features**: New TypeScript syntax and features
- **Framework Integration**: Angular, Vue, Svelte components
- **Build Tool Integration**: Webpack, Vite, Rollup compatibility
- **Edge Cases**: Unicode, very large files, circular dependencies

### Example New Test Case
```typescript
// test-input/decorators.ts - Testing decorator support
function logged(target: any, propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;
    descriptor.value = function(...args: any[]) {
        console.log(`Calling ${propertyKey} with args:`, args);
        return originalMethod.apply(this, args);
    };
}

class Calculator {
    @logged
    add(a: number, b: number): number {
        return a + b;
    }
}
```

## 🤝 Contributing Test Cases

When adding new test files:
1. **Clear Purpose**: Each file should test specific features
2. **Good Documentation**: Comments explaining the test scenario
3. **Realistic Examples**: Use patterns from real-world code
4. **Error Cases**: Include both positive and negative test cases

This test suite demonstrates thorough testing practices and attention to quality - key traits that interviewers look for in senior backend engineers.

---

**Remember**: These test files aren't just for validation - they're conversation starters about testing strategies, quality assurance, and tool development best practices.