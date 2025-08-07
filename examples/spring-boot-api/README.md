# Spring Boot API Example

This example demonstrates how to use Roelang to generate a Spring Boot REST API with JPA support.

## Features

- Spring Boot application with embedded server
- JPA entities with automatic ID generation
- Repository pattern with Spring Data JPA
- Service layer with business logic
- REST controllers with full CRUD operations
- Database operations using Roelang's `db` syntax

## Configuration

The `roeconfig.json` file configures the project to use:
- Target: `java`
- Framework: `spring`

## Files

- `user_management.roe` - Main API definition with REST endpoints
- `user_service.roe` - Service module with database operations

## Generated Code

When compiled with `roe compile`, this generates:
- Spring Boot Application class
- JPA Entity classes
- Repository interfaces
- Service classes with `@Service` annotation
- REST controllers with `@RestController` annotation

## Current Status

The Spring Boot framework support is partially implemented. Currently, it generates:
- ✅ Spring Boot main application class with `@SpringBootApplication`
- ✅ All necessary Spring Boot imports
- ⚠️ Entity, Repository, and Service classes are generated internally but not yet written to separate files

See `EXPECTED_OUTPUT.md` for the complete project structure that will be generated in the full implementation.

## Usage

1. Compile the Roelang file:
```bash
roe compile simple_spring_example.roe
```

2. Check the generated output:
```bash
cat build/simple_spring_example.java
```

The current output is a single Java file with the Spring Boot application class. Full multi-file project generation is planned for future enhancement.

## API Endpoints

The example generates these REST endpoints:
- `GET /user/{id}` - Get user by ID
- `POST /user` - Create new user
- `PUT /user/{id}` - Update existing user
- `DELETE /user/{id}` - Delete user