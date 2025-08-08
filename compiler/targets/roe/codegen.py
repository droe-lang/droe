"""Rust code generator for native RoeVM target with Axum and database support."""

import json
from typing import Dict, List, Any, Optional
from pathlib import Path
from ...codegen_base import BaseCodeGenerator
from ...ast import *


class RoeCodeGenerator(BaseCodeGenerator):
    """Generates Rust code with Axum for HTTP server and configurable database support."""
    
    def __init__(self, source_file_path: str = None, is_main_file: bool = False, 
                 framework: str = "axum", package: Optional[str] = None, 
                 database: Optional[Dict[str, Any]] = None):
        super().__init__()
        self.source_file_path = source_file_path
        self.is_main_file = is_main_file
        self.framework = framework
        self.package = package or "roelang_app"
        self.database = database or {}
        
        # Track discovered features
        self.has_serve_endpoints = False
        self.has_database_ops = False
        self.data_structures = {}
        self.serve_endpoints = []
        self.modules = {}
        
        # Database configuration
        self.db_type = self.database.get('type', 'postgres')  # Default to postgres
        self.db_url = self.database.get('url', '')
        
    def generate(self, node: ASTNode) -> Dict[str, Any]:
        """Generate complete Rust project with Axum and database support."""
        if isinstance(node, Program):
            # Analyze the program to understand what features are used
            self._analyze_program(node)
            
            # Generate project structure
            files = {}
            
            # Generate Cargo.toml
            files['Cargo.toml'] = self._generate_cargo_toml()
            
            # Generate main.rs with Axum server
            files['src/main.rs'] = self._generate_main_rs()
            
            # Generate models if data structures exist
            if self.data_structures:
                files['src/models.rs'] = self._generate_models()
                
            # Generate handlers for serve endpoints
            if self.serve_endpoints:
                files['src/handlers.rs'] = self._generate_handlers()
                
            # Generate database module if database operations exist
            if self.has_database_ops:
                files['src/db.rs'] = self._generate_database_module()
                
            # Generate lib.rs to tie modules together
            files['src/lib.rs'] = self._generate_lib_rs()
            
            return {
                'files': files,
                'project_root': self.package.replace('.', '_').replace('-', '_')
            }
        
        return {'files': {}, 'project_root': self.package}
    
    def _analyze_program(self, program: Program):
        """Analyze the program to discover features and structures."""
        for stmt in program.statements:
            self._analyze_statement(stmt)
            
    def _analyze_statement(self, stmt: ASTNode):
        """Recursively analyze statements to discover features."""
        if isinstance(stmt, DataDefinition):
            self.data_structures[stmt.name] = stmt
            self.has_database_ops = True  # Assume data structures will be persisted
            
        elif isinstance(stmt, ServeStatement):
            self.has_serve_endpoints = True
            self.serve_endpoints.append(stmt)
            # Check serve body for database operations
            for body_stmt in stmt.body:
                self._analyze_statement(body_stmt)
            
        elif isinstance(stmt, DatabaseStatement):
            self.has_database_ops = True
            
        elif isinstance(stmt, ModuleDefinition):
            self.modules[stmt.name] = stmt
            # Recursively analyze module body
            for module_stmt in stmt.body:
                self._analyze_statement(module_stmt)
                
        elif isinstance(stmt, IncludeStatement):
            # Track included modules
            pass
            
    def _generate_cargo_toml(self) -> str:
        """Generate Cargo.toml with dependencies based on used features."""
        deps = {
            'tokio': '{ version = "1.0", features = ["full"] }',
            'axum': '"0.6"',
            'serde': '{ version = "1.0", features = ["derive"] }',
            'serde_json': '"1.0"',
            'tower': '"0.4"',
            'tower-http': '{ version = "0.4", features = ["cors"] }',
            'tracing': '"0.1"',
            'tracing-subscriber': '"0.3"',
        }
        
        # Add database dependencies based on configuration
        if self.has_database_ops:
            if self.db_type == 'postgres':
                deps['sqlx'] = '{ version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }'
                deps['uuid'] = '{ version = "1", features = ["v4", "serde"] }'
                deps['chrono'] = '{ version = "0.4", features = ["serde"] }'
            elif self.db_type == 'mysql':
                deps['sqlx'] = '{ version = "0.7", features = ["runtime-tokio-rustls", "mysql", "uuid", "chrono"] }'
                deps['uuid'] = '{ version = "1", features = ["v4", "serde"] }'
                deps['chrono'] = '{ version = "0.4", features = ["serde"] }'
            elif self.db_type == 'sqlite':
                deps['sqlx'] = '{ version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "uuid", "chrono"] }'
                deps['uuid'] = '{ version = "1", features = ["v4", "serde"] }'
                deps['chrono'] = '{ version = "0.4", features = ["serde"] }'
            elif self.db_type == 'oracle':
                deps['rust-oracle'] = '"0.5"'
            elif self.db_type == 'mssql':
                deps['tiberius'] = '{ version = "0.12", features = ["rustls"] }'
                deps['tokio-util'] = '{ version = "0.7", features = ["compat"] }'
            elif self.db_type == 'mongodb':
                deps['mongodb'] = '"2"'
                deps['bson'] = '{ version = "2", features = ["chrono-0_4"] }'
        
        cargo_content = f'''[package]
name = "{self.package.replace('.', '_').replace('-', '_')}"
version = "0.1.0"
edition = "2021"

[dependencies]
'''
        
        for dep, version in deps.items():
            cargo_content += f'{dep} = {version}\n'
            
        return cargo_content
    
    def _generate_main_rs(self) -> str:
        """Generate main.rs with Axum server setup."""
        content = '''use axum::{
    Router,
    routing::{get, post, put, delete},
    response::Json,
    extract::{Path, State},
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::Arc;

mod models;
mod handlers;
'''
        
        if self.has_database_ops:
            content += 'mod db;\n'
            content += 'use db::Database;\n'
            
        content += '''
#[derive(Clone)]
pub struct AppState {
'''
        
        if self.has_database_ops:
            content += '    db: Arc<Database>,\n'
            
        content += '''}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
'''
        
        if self.has_database_ops:
            content += f'''    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "{self._get_default_db_url()}".to_string());
    let db = Arc::new(Database::new(&database_url).await.expect("Failed to connect to database"));
    
'''
        
        content += '''    // Build application state
    let state = AppState {
'''
        
        if self.has_database_ops:
            content += '        db,\n'
            
        content += '''    };
    
    // Build router
    let app = Router::new()
'''
        
        # Add routes for serve endpoints
        for endpoint in self.serve_endpoints:
            method = endpoint.method.lower()
            path = endpoint.endpoint
            handler_name = self._get_handler_name(endpoint)
            
            content += f'        .route("{path}", {method}(handlers::{handler_name}))\n'
            
        content += '''        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
'''
        return content
    
    def _generate_models(self) -> str:
        """Generate models.rs with data structures."""
        content = '''use serde::{Deserialize, Serialize};
'''
        
        if self.has_database_ops and self.db_type in ['postgres', 'mysql', 'sqlite']:
            content += 'use sqlx::FromRow;\n'
            if any(self._has_auto_id(ds) for ds in self.data_structures.values()):
                content += 'use uuid::Uuid;\n'
            if any(self._has_datetime(ds) for ds in self.data_structures.values()):
                content += 'use chrono::{DateTime, Utc};\n'
                
        content += '\n'
        
        # Generate structs for each data definition
        for name, data_def in self.data_structures.items():
            content += f'#[derive(Debug, Clone, Serialize, Deserialize'
            
            if self.has_database_ops and self.db_type in ['postgres', 'mysql', 'sqlite']:
                content += ', FromRow'
                
            content += ')]\n'
            content += f'pub struct {name} {{\n'
            
            for field in data_def.fields:
                rust_type = self._roe_type_to_rust(field.type, field.annotations)
                field_name = self._to_snake_case(field.name)
                
                # Handle optional fields
                if 'optional' in field.annotations:
                    rust_type = f'Option<{rust_type}>'
                    
                content += f'    pub {field_name}: {rust_type},\n'
                
            content += '}\n\n'
            
        return content
    
    def _generate_handlers(self) -> str:
        """Generate handlers.rs with endpoint implementations."""
        content = '''use axum::{
    response::{Json, Response, IntoResponse},
    extract::{Path, State, Query},
    http::StatusCode,
};
use serde_json::json;
use crate::{AppState, models::*};
'''
        
        if self.has_database_ops:
            content += 'use crate::db::Database;\n'
            
        content += '\n'
        
        # Generate handler functions for each serve endpoint
        for endpoint in self.serve_endpoints:
            handler_name = self._get_handler_name(endpoint)
            content += f'pub async fn {handler_name}(\n'
            
            # Add parameters based on endpoint
            if endpoint.params:
                for param in endpoint.params:
                    content += f'    Path({param}): Path<String>,\n'
                    
            if endpoint.accept_type:
                content += f'    Json(payload): Json<{endpoint.accept_type}>,\n'
                
            content += '    State(state): State<AppState>,\n'
            content += ') -> impl IntoResponse {\n'
            
            # Generate handler body based on endpoint body
            content += self._generate_handler_body(endpoint)
            
            content += '}\n\n'
            
        return content
    
    def _generate_database_module(self) -> str:
        """Generate database module with connection and query functions."""
        if self.db_type in ['postgres', 'mysql', 'sqlite']:
            return self._generate_sqlx_database()
        elif self.db_type == 'oracle':
            return self._generate_oracle_database()
        elif self.db_type == 'mssql':
            return self._generate_mssql_database()
        elif self.db_type == 'mongodb':
            return self._generate_mongodb_database()
        else:
            return '// Unsupported database type\n'
            
    def _generate_sqlx_database(self) -> str:
        """Generate SQLx-based database module."""
        content = f'''use sqlx::{{{self.db_type.capitalize()}Pool, Pool, {self.db_type}::{self.db_type.capitalize()}}};
use crate::models::*;

pub struct Database {{
    pool: Pool<{self.db_type.capitalize()}>,
}}

impl Database {{
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {{
        let pool = {self.db_type.capitalize()}Pool::connect(database_url).await?;
        Ok(Self {{ pool }})
    }}
    
    pub fn pool(&self) -> &Pool<{self.db_type.capitalize()}> {{
        &self.pool
    }}
'''
        
        # Generate CRUD methods for each data structure
        for name, data_def in self.data_structures.items():
            table_name = self._to_snake_case(name) + 's'  # Pluralize
            
            # Find operation
            content += f'''
    pub async fn find_{self._to_snake_case(name)}_by_id(&self, id: &str) -> Result<Option<{name}>, sqlx::Error> {{
        let result = sqlx::query_as::<_, {name}>(
            "SELECT * FROM {table_name} WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }}
    
    pub async fn find_all_{self._to_snake_case(name)}s(&self) -> Result<Vec<{name}>, sqlx::Error> {{
        let results = sqlx::query_as::<_, {name}>(
            "SELECT * FROM {table_name}"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(results)
    }}
    
    pub async fn create_{self._to_snake_case(name)}(&self, data: {name}) -> Result<{name}, sqlx::Error> {{
        // Implementation depends on specific fields
        todo!("Implement create")
    }}
    
    pub async fn update_{self._to_snake_case(name)}(&self, id: &str, data: {name}) -> Result<{name}, sqlx::Error> {{
        // Implementation depends on specific fields
        todo!("Implement update")
    }}
    
    pub async fn delete_{self._to_snake_case(name)}(&self, id: &str) -> Result<(), sqlx::Error> {{
        sqlx::query("DELETE FROM {table_name} WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }}
'''
            
        content += '}\n'
        return content
    
    def _generate_oracle_database(self) -> str:
        """Generate Oracle database module using rust-oracle."""
        return '''use oracle::{Connection, Error as OracleError};
use crate::models::*;

pub struct Database {
    conn_string: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, OracleError> {
        Ok(Self {
            conn_string: database_url.to_string(),
        })
    }
    
    pub fn get_connection(&self) -> Result<Connection, OracleError> {
        Connection::connect(&self.conn_string, "", "")
    }
    
    // Add CRUD methods here
}
'''
    
    def _generate_mssql_database(self) -> str:
        """Generate MS SQL database module using tiberius."""
        return '''use tiberius::{Client, Config, AuthMethod, error::Error as TiberiusError};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use crate::models::*;

pub struct Database {
    config: Config,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, TiberiusError> {
        let config = Config::from_ado_string(database_url)?;
        Ok(Self { config })
    }
    
    pub async fn get_client(&self) -> Result<Client<tokio_util::compat::Compat<TcpStream>>, TiberiusError> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        let tcp = tcp.compat_write();
        let client = Client::connect(self.config.clone(), tcp).await?;
        Ok(client)
    }
    
    // Add CRUD methods here
}
'''
    
    def _generate_mongodb_database(self) -> str:
        """Generate MongoDB database module."""
        return '''use mongodb::{Client, Database as MongoDatabase, Collection, error::Error as MongoError};
use crate::models::*;

pub struct Database {
    client: Client,
    db: MongoDatabase,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, MongoError> {
        let client = Client::with_uri_str(database_url).await?;
        let db = client.database("roelang_app");
        Ok(Self { client, db })
    }
    
    pub fn collection<T>(&self, name: &str) -> Collection<T> {
        self.db.collection(name)
    }
    
    // Add CRUD methods here
}
'''
    
    def _generate_lib_rs(self) -> str:
        """Generate lib.rs to export modules."""
        content = 'pub mod models;\npub mod handlers;\n'
        
        if self.has_database_ops:
            content += 'pub mod db;\n'
            
        content += '\npub use models::*;\n'
        return content
    
    def _generate_handler_body(self, endpoint: ServeStatement) -> str:
        """Generate the body of a handler function based on serve statement."""
        # This is a simplified implementation
        # In a real implementation, we would parse the endpoint body
        # and generate appropriate Rust code
        
        content = '    // Handler implementation\n'
        
        if endpoint.response_action:
            content += '    // Call action and return response\n'
            
        content += '    Json(json!({"message": "Endpoint not fully implemented"}))\n'
        
        return content
    
    def _get_handler_name(self, endpoint: ServeStatement) -> str:
        """Generate a handler function name from endpoint."""
        method = endpoint.method.lower()
        path_parts = endpoint.endpoint.strip('/').split('/')
        
        # Filter out parameter parts
        path_parts = [p for p in path_parts if not p.startswith(':')]
        
        if path_parts:
            return f'{method}_{"_".join(path_parts)}'
        else:
            return f'{method}_root'
            
    def _roe_type_to_rust(self, roe_type: str, annotations: List[str]) -> str:
        """Convert Roe type to Rust type."""
        type_map = {
            'text': 'String',
            'int': 'i32',
            'decimal': 'f64',
            'flag': 'bool',
            'date': 'DateTime<Utc>',
            'datetime': 'DateTime<Utc>',
        }
        
        # Handle auto-generated IDs
        if 'key' in annotations and 'auto' in annotations:
            return 'Uuid'
            
        return type_map.get(roe_type, 'String')
    
    def _to_snake_case(self, name: str) -> str:
        """Convert CamelCase to snake_case."""
        result = []
        for i, char in enumerate(name):
            if char.isupper() and i > 0:
                result.append('_')
            result.append(char.lower())
        return ''.join(result)
    
    def _has_auto_id(self, data_def: DataDefinition) -> bool:
        """Check if data definition has auto-generated ID field."""
        for field in data_def.fields:
            if 'key' in field.annotations and 'auto' in field.annotations:
                return True
        return False
    
    def _has_datetime(self, data_def: DataDefinition) -> bool:
        """Check if data definition has datetime fields."""
        for field in data_def.fields:
            if field.type in ['date', 'datetime']:
                return True
        return False
    
    def _get_default_db_url(self) -> str:
        """Get default database URL based on database type."""
        urls = {
            'postgres': 'postgresql://localhost/roelang_db',
            'mysql': 'mysql://localhost/roelang_db',
            'sqlite': 'sqlite:roelang.db',
            'oracle': 'localhost:1521/XE',
            'mssql': 'server=localhost;database=roelang_db',
            'mongodb': 'mongodb://localhost:27017',
        }
        return urls.get(self.db_type, 'postgresql://localhost/roelang_db')
    
    def emit_statement(self, stmt: ASTNode) -> str:
        """Emit Rust code for a statement (required by base class)."""
        # This is handled by the generate method for project generation
        return ""
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit Rust code for an expression (required by base class)."""
        # This is handled by the generate method for project generation
        return ""