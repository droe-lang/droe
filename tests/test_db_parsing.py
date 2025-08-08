#!/usr/bin/env python3
"""Test database DSL parsing."""

import sys
from pathlib import Path

# Add compiler to path
sys.path.insert(0, str(Path(__file__).parent / "compiler"))

from compiler.parser.core import Parser

def test_db_parsing():
    """Test basic database operations parsing."""
    
    # Test data with annotations
    source_code = '''
module UserModule

data User
    id is text key auto
    name is text required
    email is text required unique
    createdAt is datetime auto
end data

action getUser with id which is text gives User
    db find User where id equals id
    expect user
    give user
end action

action getAllUsers gives list of User
    db find all User
    expect users
    give users
end action

action createUser with name which is text, email which is text gives User
    db create User with name is name and email is email
    return id into userId
    give User with id is userId, name is name, email is email
end action

action updateUser with id which is text, name which is text gives User
    db update User where id equals id set name to name
    expect user
    give user
end action

action deleteUser with id which is text
    db delete User where id equals id
    expect success
end action

end module
'''

    parser = Parser()
    ast = parser.parse(source_code)
    
    print("✅ Database DSL parsing successful!")
    print(f"📊 AST contains {len(ast.statements)} statements")
    
    # Find the module
    module_stmt = None
    for stmt in ast.statements:
        if hasattr(stmt, 'name') and stmt.name == 'UserModule':
            module_stmt = stmt
            break
    
    if module_stmt:
        print(f"📦 Found module: {module_stmt.name}")
        
        # Check data definitions
        data_defs = [stmt for stmt in module_stmt.body if hasattr(stmt, 'fields')]
        for data_def in data_defs:
            print(f"🗂️  Data: {data_def.name}")
            for field in data_def.fields:
                annotations_str = f" {' '.join(field.annotations)}" if field.annotations else ""
                print(f"   - {field.name}: {field.type}{annotations_str}")
        
        # Check database operations in actions
        actions = [stmt for stmt in module_stmt.body if hasattr(stmt, 'body')]
        for action in actions:
            print(f"⚡ Action: {action.name}")
            for stmt in action.body:
                if hasattr(stmt, 'operation'):  # DatabaseStatement
                    print(f"   🗄️  db {stmt.operation} {stmt.entity_name}")
    
    return True

if __name__ == '__main__':
    try:
        test_db_parsing()
        print("\n🎉 All database parsing tests passed!")
    except Exception as e:
        print(f"\n❌ Database parsing test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)