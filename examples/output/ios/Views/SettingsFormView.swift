import SwiftUI

struct SettingsFormView: View {
    
    var body: some View {
        Form {
            Section {
                Text("App Settings")
                    .font(.title2)
                    .fontWeight(.bold)
            }
            
        }
        .navigationTitle("Settings")
    }
    
    
}

struct SettingsFormView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsFormView()
    }
}