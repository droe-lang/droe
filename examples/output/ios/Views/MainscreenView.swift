import SwiftUI

struct MainscreenView: View {
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("PhotoShare App")
                    .font(.largeTitle)
                    .padding(.bottom, 8)
                
            }
            .padding()
        }
    }
    
}

struct MainscreenView_Previews: PreviewProvider {
    static var previews: some View {
        MainscreenView()
    }
}