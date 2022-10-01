import { motion } from "framer-motion";
import { useState } from "react";
import Typewriter from 'typewriter-effect';
import Card from "../card";

const LandingPage = () => {
    return (
        <FondoImagen />
    );
}

const FondoImagen = () => {
    return (
        <main className="flex justify-center bg-white font-outfit-light h-screen w-screen p-4 sm:p-10 text-white">
            <motion.div
                initial={{ opacity: 0, scaleY: 0, y: '50vh' }} animate={{ opacity: 1, scaleY: 1, transition: { duration: 0.8 }, y: '0vh' }}
                className="bg-gradient-to-r from-blue-400 via-purple-500 to-orange-600  animate-gradient-x
            flex items-center justify-center w-full h-full
                text-[20vw] font-outfit-light overflow-clip">
                <Typewriter
                    options={{ delay: 70, deleteSpeed: 27, loop: true }}
                    onInit={(typewriter) => {
                        typewriter
                            .typeString('¡Hola!')
                            .pauseFor(3000)
                            .deleteAll()
                            .typeString('Oi!')
                            .pauseFor(3000)
                            .deleteAll()
                            .typeString('Hello!')
                            .pauseFor(3000)
                            .deleteAll()
                            .typeString('Bon dia!')
                            .pauseFor(3000)
                            .stop()
                            .start();
                    }}
                />
            </motion.div>
        </main >
    );
}

export default LandingPage;