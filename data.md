 Frontend:
  cd frontend
  cp .env.example .env      # set PUBLIC_API_URL if needed
  npm run dev               # → http://localhost:4321                                                                                                 
  
  Backend:                                                                                                                                            
  cd backend      
  cp .env.example .env      # fill SMTP_PASS (Gmail App Password)                                                                                     
  cargo run                 # → http://localhost:8080            
                                                                                                                                                      
  Gmail App Password (needed for the contact form to send emails):
  1. Go to myaccount.google.com → Security → 2-Step Verification → App Passwords                                                                      
  2. Generate one for "Incopter backend"                                                                                                              
  3. Paste it in backend/.env as SMTP_PASS                                                                                                            
                                                                                                                                                      
  ---                                                                                                                                                 
  Next steps whenever you're ready: add your logo, photos, social media links, and I can set up deployment to a VPS or cloud for incopter.gr.