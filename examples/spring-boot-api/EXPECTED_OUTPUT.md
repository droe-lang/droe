# Expected Spring Boot Output Structure

When the Spring Boot framework is fully implemented, compiling a Roelang file with `framework: "spring"` should generate a complete Spring Boot project structure:

## Project Structure
```
build/
└── spring-boot-project/
    ├── pom.xml                                 # Maven configuration
    ├── src/
    │   └── main/
    │       ├── java/
    │       │   └── com/
    │       │       └── example/
    │       │           └── app/
    │       │               ├── Application.java        # Main Spring Boot class
    │       │               ├── entity/
    │       │               │   └── Product.java       # JPA Entity
    │       │               ├── repository/
    │       │               │   └── ProductRepository.java
    │       │               ├── service/
    │       │               │   └── ProductService.java
    │       │               └── controller/
    │       │                   └── ProductController.java
    │       └── resources/
    │           └── application.properties     # Spring Boot configuration
    └── README.md
```

## Generated Files

### 1. Application.java
```java
package com.example.app;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

### 2. entity/Product.java
```java
package com.example.app.entity;

import jakarta.persistence.*;

@Entity
@Table(name = "products")
public class Product {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(name = "name")
    private String name;
    
    @Column(name = "price")
    private Double price;
    
    @Column(name = "in_stock")
    private Boolean inStock;
    
    // Getters and setters...
}
```

### 3. repository/ProductRepository.java
```java
package com.example.app.repository;

import com.example.app.entity.Product;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

@Repository
public interface ProductRepository extends JpaRepository<Product, Long> {
    // Custom queries can be added here
}
```

### 4. service/ProductService.java
```java
package com.example.app.service;

import com.example.app.entity.Product;
import com.example.app.repository.ProductRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import java.util.List;
import java.util.Optional;

@Service
public class ProductService {
    
    @Autowired
    private ProductRepository productRepository;
    
    public List<Product> findAllProducts() {
        return productRepository.findAll();
    }
    
    public Product findProductById(String productId) {
        Optional<Product> result = productRepository.findById(Long.parseLong(productId));
        return result.orElse(null);
    }
    
    public Product saveProduct(String productName, Double productPrice) {
        Product product = new Product();
        product.setName(productName);
        product.setPrice(productPrice);
        product.setInStock(true);
        return productRepository.save(product);
    }
}
```

### 5. pom.xml
```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.1.0</version>
    </parent>
    
    <groupId>com.example</groupId>
    <artifactId>roelang-spring-app</artifactId>
    <version>1.0.0</version>
    
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
        </dependency>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-data-jpa</artifactId>
        </dependency>
        <dependency>
            <groupId>com.h2database</groupId>
            <artifactId>h2</artifactId>
            <scope>runtime</scope>
        </dependency>
    </dependencies>
    
    <build>
        <plugins>
            <plugin>
                <groupId>org.springframework.boot</groupId>
                <artifactId>spring-boot-maven-plugin</artifactId>
            </plugin>
        </plugins>
    </build>
</project>
```

## Current Limitation

Currently, the implementation only generates the main Application class in a single file. The full multi-file project generation with proper package structure is planned for future enhancement.

## Running the Generated Project

Once fully implemented, you would be able to:

1. Compile with Roelang:
   ```bash
   roe compile simple_spring_example.roe
   ```

2. Navigate to the generated project:
   ```bash
   cd build/spring-boot-project
   ```

3. Run with Maven:
   ```bash
   mvn spring-boot:run
   ```

4. Or build and run the JAR:
   ```bash
   mvn clean package
   java -jar target/roelang-spring-app-1.0.0.jar
   ```

The application would start on http://localhost:8080 with REST endpoints available.